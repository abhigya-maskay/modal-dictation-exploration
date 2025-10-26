mod backend;
mod renderer;
mod state;
mod wayland;

pub use backend::{OverlayBackend, MockOverlayBackend, FailingMockBackend};
pub use renderer::OverlayColor;
pub use state::{OverlayRenderState, ReconnectionState};
pub use wayland::{OverlayPosition, WaylandOverlay};

use crate::activation::ActivationManager;
use crate::config::ConfigManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

/// Manages the overlay indicator, subscribing to state and config changes
pub struct OverlayManager {
    state: Arc<tokio::sync::Mutex<OverlayRenderState>>,
    reconnection_state: Arc<tokio::sync::Mutex<ReconnectionState>>,
    task_handle: JoinHandle<()>,
}

impl OverlayManager {
    /// Creates a new overlay manager with a custom backend factory and spawns the update task
    ///
    /// # Arguments
    /// * `config_manager` - Configuration manager for live reload support
    /// * `activation_manager` - Activation state manager for state updates
    /// * `backend_factory` - Factory function to create overlay backends
    ///
    /// # Returns
    /// A new OverlayManager that automatically tracks state and config changes.
    /// Always succeeds by using fallback colors for invalid config values.
    pub fn new_with_factory<F>(
        config_manager: &ConfigManager,
        activation_manager: &Arc<ActivationManager>,
        backend_factory: F,
    ) -> Self
    where
        F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
    {
        let initial_config = config_manager.current();
        let initial_state = activation_manager.current_state();

        let render_state = OverlayRenderState::new(initial_state, initial_config.overlay.clone());

        let state = Arc::new(tokio::sync::Mutex::new(render_state));
        let reconnection_state = Arc::new(tokio::sync::Mutex::new(ReconnectionState::new()));

        let state_clone = state.clone();
        let reconnection_clone = reconnection_state.clone();

        let mut config_rx = config_manager.subscribe();
        let mut activation_rx = activation_manager.subscribe();

        let backend_factory = Arc::new(backend_factory);

        let task_handle = tokio::spawn(async move {
            let mut overlay: Option<Box<dyn OverlayBackend>> = None;
            let mut last_color: Option<OverlayColor> = None;

            let position_str = {
                let state = state_clone.lock().await;
                state.config.position.clone()
            };

            let position = match OverlayPosition::from_str(&position_str) {
                Ok(pos) => pos,
                Err(e) => {
                    tracing::warn!("Invalid overlay position: {}, using default (top-right)", e);
                    OverlayPosition::TopRight
                }
            };

            match backend_factory(position) {
                Ok(mut backend) => {
                    if let Err(e) = backend.connect().await {
                        tracing::warn!("Failed to connect to Wayland compositor: {}", e);
                        let mut recon = reconnection_clone.lock().await;
                        recon.record_failure();
                        let mut state = state_clone.lock().await;
                        state.set_error(true);
                    } else {
                        overlay = Some(backend);
                        tracing::info!("Overlay connected to backend");
                        let mut state = state_clone.lock().await;
                        state.set_error(false);
                        drop(state);

                        let color = {
                            let state = state_clone.lock().await;
                            state.current_color()
                        };

                        if let Some(overlay_ref) = &mut overlay {
                            if let Err(e) = overlay_ref.update_color(color).await {
                                tracing::warn!(
                                    "Failed to update overlay color after initial connection: {}",
                                    e
                                );
                                let mut state = state_clone.lock().await;
                                state.set_error(true);
                                let error_color = state.current_color();
                                drop(state);

                                if let Err(_) = overlay_ref.update_color(error_color).await {
                                    overlay = None;
                                    let mut recon = reconnection_clone.lock().await;
                                    recon.record_failure();
                                }
                            } else {
                                last_color = Some(color);
                                let mut state = state_clone.lock().await;
                                state.set_error(false);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create Wayland overlay: {}", e);
                    let mut recon = reconnection_clone.lock().await;
                    recon.record_failure();
                    let mut state = state_clone.lock().await;
                    state.set_error(true);
                }
            }

            let mut health_check_interval = tokio::time::interval(Duration::from_secs(2));

            loop {
                tokio::select! {
                    _ = health_check_interval.tick() => {
                        if let Some(overlay_ref) = &mut overlay {
                            let current_color = {
                                let state = state_clone.lock().await;
                                state.current_color()
                            };

                            if let Err(e) = overlay_ref.update_color(current_color).await {
                                tracing::debug!("Health check detected broken overlay: {}", e);
                                let mut state = state_clone.lock().await;
                                state.set_error(true);
                                let error_color = state.current_color();
                                drop(state);

                                if let Err(_) = overlay_ref.update_color(error_color).await {
                                    overlay = None;
                                    let mut recon = reconnection_clone.lock().await;
                                    recon.record_failure();
                                }
                            }
                        }
                    }

                    _ = config_rx.changed() => {
                        let new_full_config = config_rx.borrow().clone();
                        let new_overlay_config = new_full_config.overlay.clone();
                        let position_str = new_overlay_config.position.clone();

                        let mut state = state_clone.lock().await;
                        if let Err(e) = state.update_config(new_overlay_config) {
                            tracing::warn!("Failed to update overlay config: {}", e);
                            state.set_error(true);
                        } else {
                            tracing::info!("Overlay config updated");

                            let new_position = match OverlayPosition::from_str(&position_str) {
                                Ok(pos) => pos,
                                Err(e) => {
                                    tracing::warn!("Invalid overlay position: {}", e);
                                    OverlayPosition::TopRight
                                }
                            };

                            if let Some(current_overlay) = &overlay {
                                if current_overlay.position() != new_position {
                                    tracing::info!("Overlay position changed, reconnecting...");
                                    overlay = None;
                                }
                            }
                        }

                        drop(state);

                        let color = {
                            let state = state_clone.lock().await;
                            state.current_color()
                        };

                        if let Some(overlay_ref) = &mut overlay {
                            if let Err(e) = overlay_ref.update_color(color).await {
                                tracing::warn!("Failed to update overlay color: {}", e);
                                let mut state = state_clone.lock().await;
                                state.set_error(true);
                                let error_color = state.current_color();
                                drop(state);

                                if let Err(_) = overlay_ref.update_color(error_color).await {
                                    overlay = None;
                                }
                            } else {
                                last_color = Some(color);
                                let mut state = state_clone.lock().await;
                                state.set_error(false);
                            }
                        }
                    }

                    _ = activation_rx.changed() => {
                        let (new_state, _transition) = *activation_rx.borrow();
                        let mut state = state_clone.lock().await;
                        state.update_system_state(new_state);
                        drop(state);

                        let color = {
                            let state = state_clone.lock().await;
                            state.current_color()
                        };

                        if Some(color) != last_color {
                            if let Some(overlay_ref) = &mut overlay {
                                if let Err(e) = overlay_ref.update_color(color).await {
                                    tracing::warn!("Failed to update overlay color: {}", e);
                                    let mut state = state_clone.lock().await;
                                    state.set_error(true);
                                    let error_color = state.current_color();
                                    drop(state);

                                    if let Err(_) = overlay_ref.update_color(error_color).await {
                                        overlay = None;
                                        let mut recon = reconnection_clone.lock().await;
                                        recon.record_failure();
                                    }
                                } else {
                                    last_color = Some(color);
                                    let mut recon = reconnection_clone.lock().await;
                                    recon.reset();
                                    let mut state = state_clone.lock().await;
                                    state.set_error(false);
                                }
                            }
                        }
                    }

                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        if overlay.is_none() {
                            let recon = reconnection_clone.lock().await;
                            if recon.should_retry() {
                                drop(recon);

                                let position_str = {
                                    let state = state_clone.lock().await;
                                    state.config.position.clone()
                                };

                                let position = match OverlayPosition::from_str(&position_str) {
                                    Ok(pos) => pos,
                                    Err(e) => {
                                        tracing::warn!(
                                            "Invalid overlay position during reconnect: {}",
                                            e
                                        );
                                        OverlayPosition::TopRight
                                    }
                                };

                                match backend_factory(position) {
                                    Ok(mut backend) => {
                                        match backend.connect().await {
                                            Ok(()) => {
                                                overlay = Some(backend);

                                                let color = {
                                                    let state = state_clone.lock().await;
                                                    state.current_color()
                                                };

                                                if let Some(overlay_ref) = &mut overlay {
                                                    if let Err(e) = overlay_ref.update_color(color).await
                                                    {
                                                        tracing::warn!(
                                                            "Failed to update color after reconnect: {}",
                                                            e
                                                        );
                                                        let mut state = state_clone.lock().await;
                                                        state.set_error(true);
                                                        let error_color = state.current_color();
                                                        drop(state);

                                                        if let Err(_) = overlay_ref.update_color(error_color).await {
                                                            overlay = None;
                                                        }
                                                    } else {
                                                        last_color = Some(color);
                                                        let mut recon = reconnection_clone.lock().await;
                                                        recon.reset();
                                                        let mut state = state_clone.lock().await;
                                                        state.set_error(false);
                                                        tracing::info!("Overlay reconnected successfully");
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                tracing::debug!(
                                                    "Reconnection attempt failed: {}",
                                                    e
                                                );
                                                let mut recon = reconnection_clone.lock().await;
                                                let wait_time = recon.record_failure();
                                                let mut state = state_clone.lock().await;
                                                state.set_error(true);
                                                tracing::debug!(
                                                    "Will retry in {:?}",
                                                    wait_time
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::debug!("Failed to create overlay during reconnect: {}", e);
                                        let mut recon = reconnection_clone.lock().await;
                                        recon.record_failure();
                                        let mut state = state_clone.lock().await;
                                        state.set_error(true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            state,
            reconnection_state,
            task_handle,
        }
    }

    /// Creates a new overlay manager with the default Wayland backend
    ///
    /// # Arguments
    /// * `config_manager` - Configuration manager for live reload support
    /// * `activation_manager` - Activation state manager for state updates
    ///
    /// # Returns
    /// A new OverlayManager that uses Wayland overlay backend.
    /// Always succeeds by using fallback colors for invalid config values.
    pub fn new_with_wayland(
        config_manager: &ConfigManager,
        activation_manager: &Arc<ActivationManager>,
    ) -> Self {
        Self::new_with_factory(config_manager, activation_manager, |position| {
            WaylandOverlay::new(position)
                .map(|overlay| Box::new(overlay) as Box<dyn OverlayBackend>)
        })
    }

    /// Returns the current overlay render state
    pub async fn current_state(&self) -> OverlayRenderState {
        self.state.lock().await.clone()
    }

    /// Returns whether an error is currently set
    pub async fn has_error(&self) -> bool {
        self.state.lock().await.has_error
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activation::SystemState;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_config_dir(config_content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, config_content)
            .expect("Failed to write test config");
        let path = temp_dir.path().to_path_buf();
        (temp_dir, path)
    }

    fn create_default_test_config() -> (TempDir, PathBuf) {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        create_test_config_dir(config_content)
    }

    fn create_test_overlay_manager(
        config_mgr: &ConfigManager,
        activation_mgr: &Arc<ActivationManager>,
    ) -> OverlayManager {
        OverlayManager::new_with_factory(config_mgr, activation_mgr, |position| {
            MockOverlayBackend::new(position)
                .map(|backend| Box::new(backend) as Box<dyn OverlayBackend>)
        })
    }

    #[tokio::test]
    async fn test_overlay_manager_creation_with_defaults() {
        let (_temp_dir, config_path) = create_default_test_config();
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Asleep);
        assert!(!state.has_error, "Should not have error on creation");
    }

    #[tokio::test]
    async fn test_overlay_color_selection_asleep_state() {
        let (_temp_dir, config_path) = create_default_test_config();
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Asleep);
        let color = state.current_color();
        assert_eq!(color, OverlayColor::opaque(128, 128, 128),
                   "Asleep state should use gray color");

        drop(overlay);
    }

    #[tokio::test]
    async fn test_overlay_custom_colors() {
        let config_content = r#"
[overlay]
asleep_color = "blue"
awake_color = "yellow"
error_color = "red"
position = "bottom-left"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Asleep);
        let color = state.current_color();
        assert_eq!(color, OverlayColor::opaque(0, 0, 255),
                   "Custom asleep color should be blue");
        assert_eq!(state.config.position, "bottom-left");

        drop(overlay);
    }

    #[tokio::test]
    async fn test_overlay_error_state_tracking() {
        let (_temp_dir, config_path) = create_default_test_config();
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let initial_state = overlay.current_state().await;
        assert!(!initial_state.has_error, "Should start without error");

        let has_error = overlay.has_error().await;
        assert!(!has_error, "Initial state should not have error");

        drop(overlay);
    }

    #[tokio::test]
    async fn test_overlay_state_initialization_preserves_config() {
        let config_content = r#"
[overlay]
asleep_color = "purple"
awake_color = "cyan"
error_color = "red"
position = "bottom-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.config.position, "bottom-right");
        assert_eq!(state.config.asleep_color, "purple");
        assert_eq!(state.config.awake_color, "cyan");

        assert_eq!(state.asleep_color, OverlayColor::opaque(128, 0, 128));
        assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 255));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_overlay_multiple_instances_independent_state() {
        let config_content1 = r#"
[overlay]
asleep_color = "blue"
awake_color = "yellow"
error_color = "red"
position = "top-left"
"#;
        let config_content2 = r#"
[overlay]
asleep_color = "green"
awake_color = "red"
error_color = "yellow"
position = "bottom-right"
"#;

        let (_temp_dir1, config_path1) = create_test_config_dir(config_content1);
        let (_temp_dir2, config_path2) = create_test_config_dir(config_content2);

        let config_mgr1 = ConfigManager::new_with_path(config_path1)
            .expect("Failed to create config manager 1");
        let config_mgr2 = ConfigManager::new_with_path(config_path2)
            .expect("Failed to create config manager 2");

        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay1 = create_test_overlay_manager(&config_mgr1, &activation_mgr);
        let overlay2 = create_test_overlay_manager(&config_mgr2, &activation_mgr);

        let state1 = overlay1.current_state().await;
        let state2 = overlay2.current_state().await;

        assert_eq!(state1.config.position, "top-left");
        assert_eq!(state2.config.position, "bottom-right");

        assert_eq!(state1.asleep_color, OverlayColor::opaque(0, 0, 255));
        assert_eq!(state2.asleep_color, OverlayColor::opaque(0, 255, 0));

        drop(overlay1);
        drop(overlay2);
    }

    #[tokio::test]
    async fn test_overlay_color_based_on_error_state() {
        let (_temp_dir, config_path) = create_default_test_config();
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert!(!state.has_error);

        let color = state.current_color();
        assert_eq!(color, OverlayColor::opaque(128, 128, 128));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_overlay_state_with_hex_colors() {
        let config_content = "[overlay]\nasleep_color = \"#FF00FF\"\nawake_color = \"#00FF00\"\nerror_color = \"#0000FF\"\nposition = \"top-right\"\n";
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.asleep_color, OverlayColor::opaque(255, 0, 255));
        assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 0));
        assert_eq!(state.error_color, OverlayColor::opaque(0, 0, 255));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_activation_state_change_updates_color() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Asleep);
        assert_eq!(state.current_color(), OverlayColor::opaque(128, 128, 128));

        activation_mgr.wake_via_wake_word().await;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Awake);
        assert_eq!(state.current_color(), OverlayColor::opaque(0, 255, 0));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_config_change_updates_colors() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path.clone())
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

        activation_mgr.wake_via_wake_word().await;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let state = overlay.current_state().await;
        assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 0));

        let new_config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "blue"
error_color = "red"
position = "top-right"
"#;
        let config_file_path = config_path.join("config.toml");
        std::fs::write(&config_file_path, new_config_content)
            .expect("Failed to write updated config");

        tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        let state = overlay.current_state().await;
        assert_eq!(state.awake_color, OverlayColor::opaque(0, 0, 255));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_error_state_switches_to_error_color() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
            let count = attempt_count_clone.clone();
            FailingMockBackend::new(position).map(move |backend| {
                let mut attempts = count.lock().unwrap();
                *attempts += 1;
                let attempt_num = *attempts;
                drop(attempts);

                if attempt_num == 1 {
                    Box::new(backend.fail_update_color_n_times(1000)) as Box<dyn OverlayBackend>
                } else {
                    Box::new(backend) as Box<dyn OverlayBackend>
                }
            })
        });

        tokio::time::sleep(std::time::Duration::from_millis(150)).await;

        let state = overlay.current_state().await;
        assert!(state.has_error, "Should have error after failed initial color update");
        assert_eq!(state.current_color(), OverlayColor::opaque(255, 0, 0));

        drop(overlay);
    }

    #[tokio::test]
    async fn test_reconnection_after_initial_failure() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let factory_call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let factory_call_count_clone = factory_call_count.clone();

        let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
            let mut count = factory_call_count_clone.lock().unwrap();
            *count += 1;

            FailingMockBackend::new(position)
                .map(|backend| {
                    if *count <= 2 {
                        Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
                    } else {
                        Box::new(backend) as Box<dyn OverlayBackend>
                    }
                })
        });

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let state = overlay.current_state().await;
        assert_eq!(state.system_state, SystemState::Asleep);

        drop(overlay);
    }

    #[tokio::test]
    async fn test_reconnection_exponential_backoff() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
            let count = attempt_count_clone.clone();
            FailingMockBackend::new(position)
                .map(move |backend| {
                    let mut attempts = count.lock().unwrap();
                    *attempts += 1;
                    drop(attempts);

                    Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
                })
        });

        tokio::time::sleep(std::time::Duration::from_secs(6)).await;

        let times = attempt_count.lock().unwrap();
        assert!(*times >= 3, "Should have at least 3 connection attempts, got {}", *times);

        drop(overlay);
    }

    #[tokio::test]
    async fn test_successful_reconnect_resets_backoff() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let should_fail = std::sync::Arc::new(std::sync::Mutex::new(true));
        let should_fail_clone = should_fail.clone();

        let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
            FailingMockBackend::new(position)
                .map(|backend| {
                    let fail = should_fail_clone.lock().unwrap();
                    if *fail {
                        Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
                    } else {
                        Box::new(backend) as Box<dyn OverlayBackend>
                    }
                })
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        *should_fail.lock().unwrap() = false;

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let state = overlay.current_state().await;
        assert!(!state.has_error, "Should clear error after successful reconnection");

        drop(overlay);
    }

    #[tokio::test]
    async fn test_health_check_detects_broken_overlay() {
        let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
        let (_temp_dir, config_path) = create_test_config_dir(config_content);
        let config_mgr = ConfigManager::new_with_path(config_path)
            .expect("Failed to create config manager");
        let activation_mgr = Arc::new(ActivationManager::new(300));

        let health_fail_count = std::sync::Arc::new(std::sync::Mutex::new(false));
        let health_fail_count_clone = health_fail_count.clone();

        let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
            let should_fail = *health_fail_count_clone.lock().unwrap();
            FailingMockBackend::new(position)
                .map(|backend| {
                    if should_fail {
                        Box::new(backend.fail_update_color_n_times(1)) as Box<dyn OverlayBackend>
                    } else {
                        Box::new(backend) as Box<dyn OverlayBackend>
                    }
                })
        });

        let state = overlay.current_state().await;
        assert!(!state.has_error);

        *health_fail_count.lock().unwrap() = true;

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let state = overlay.current_state().await;
        assert!(state.has_error, "Health check should detect broken overlay");

        drop(overlay);
    }

}
