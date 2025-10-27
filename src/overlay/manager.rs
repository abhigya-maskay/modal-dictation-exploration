//! Overlay manager implementation
//!
//! This module contains the `OverlayManager` which subscribes to configuration
//! and activation state changes, manages backend lifecycle (connect/reconnect),
//! and coordinates color updates with health checks and exponential backoff.

use crate::activation::ActivationManager;
use crate::config::{ConfigManager, WatcherHealth};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

use super::{OverlayBackend, OverlayColor, OverlayPosition, OverlayRenderState, ReconnectionState, WaylandOverlay, wayland};

/// Lightweight context holding shared overlay state and dependencies
///
/// Encapsulates the commonly-shared state (render state, reconnection tracking,
/// backend factory) to reduce parameter repetition across helper functions.
struct OverlayContext<F>
where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    state: Arc<tokio::sync::Mutex<OverlayRenderState>>,
    reconnection: Arc<tokio::sync::Mutex<ReconnectionState>>,
    backend_factory: Arc<F>,
}

impl<F> OverlayContext<F>
where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    fn new(
        state: Arc<tokio::sync::Mutex<OverlayRenderState>>,
        reconnection: Arc<tokio::sync::Mutex<ReconnectionState>>,
        backend_factory: Arc<F>,
    ) -> Self {
        Self {
            state,
            reconnection,
            backend_factory,
        }
    }

    fn state(&self) -> &Arc<tokio::sync::Mutex<OverlayRenderState>> {
        &self.state
    }

    fn reconnection(&self) -> &Arc<tokio::sync::Mutex<ReconnectionState>> {
        &self.reconnection
    }

    fn factory(&self) -> &Arc<F> {
        &self.backend_factory
    }
}

/// Parses an overlay position string with fallback to TopRight on error
pub fn parse_position_with_fallback(position_str: &str) -> OverlayPosition {
    match OverlayPosition::from_str(position_str) {
        Ok(pos) => pos,
        Err(e) => {
            tracing::warn!("Invalid overlay position: {}, using default (top-right)", e);
            OverlayPosition::TopRight
        }
    }
}

/// Updates overlay color with fallback error handling
///
/// Returns true if overlay is still valid, false if it was cleared
async fn try_update_color_with_fallback<F>(
    ctx: &OverlayContext<F>,
    overlay: &mut Option<Box<dyn OverlayBackend>>,
    color: OverlayColor,
    last_color: &mut Option<OverlayColor>,
    context: &str,
    record_failure_on_double_fail: bool,
    reset_reconnection_on_success: bool,
) -> bool
where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    if let Some(overlay_ref) = overlay {
        if let Err(e) = overlay_ref.update_color(color).await {
            tracing::warn!("Failed to update overlay color {}: {}", context, e);
            let mut state = ctx.state().lock().await;
            state.set_error(true);
            let error_color = state.current_color();
            drop(state);

            if let Err(_) = overlay_ref.update_color(error_color).await {
                *overlay = None;
                if record_failure_on_double_fail {
                    let mut recon = ctx.reconnection().lock().await;
                    recon.record_failure();
                }
                return false;
            }
        } else {
            *last_color = Some(color);
            if reset_reconnection_on_success {
                let mut recon = ctx.reconnection().lock().await;
                recon.reset();
            }
            let mut state = ctx.state().lock().await;
            state.set_error(false);
        }
    }
    true
}

/// Attempts to create and initialize a backend at the given position
///
/// Handles backend creation, connection, and initial color update.
/// Updates reconnection_state and render_state on success/failure.
async fn connect_and_initialize_backend<F>(
    ctx: &OverlayContext<F>,
    position: OverlayPosition,
    last_color: &mut Option<OverlayColor>,
    context: &str,
) -> Option<Box<dyn OverlayBackend>>
where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    match (ctx.factory())(position) {
        Ok(mut backend) => {
            if let Err(e) = backend.connect().await {
                tracing::warn!("Failed to connect to Wayland compositor: {}", e);
                let mut recon = ctx.reconnection().lock().await;
                recon.record_failure();
                let mut state = ctx.state().lock().await;
                state.set_error(true);
                None
            } else {
                tracing::info!("Overlay connected to backend");
                let mut state = ctx.state().lock().await;
                state.set_error(false);
                drop(state);

                let color = {
                    let state = ctx.state().lock().await;
                    state.current_color()
                };

                let mut overlay = Some(backend);
                let was_successful = try_update_color_with_fallback(
                    ctx,
                    &mut overlay,
                    color,
                    last_color,
                    context,
                    true,
                    true,
                )
                .await;

                if was_successful {
                    overlay
                } else {
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to create Wayland overlay: {}", e);
            let mut recon = ctx.reconnection().lock().await;
            recon.record_failure();
            let mut state = ctx.state().lock().await;
            state.set_error(true);
            None
        }
    }
}

/// Handles periodic health check - updates overlay color if still valid
async fn handle_health_check<F>(
    ctx: &OverlayContext<F>,
    overlay: &mut Option<Box<dyn OverlayBackend>>,
    last_color: &mut Option<OverlayColor>,
) where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    let current_color = {
        let state = ctx.state().lock().await;
        state.current_color()
    };

    try_update_color_with_fallback(
        ctx,
        overlay,
        current_color,
        last_color,
        "during health check",
        true,
        true,
    )
    .await;
}

/// Handles configuration change - updates state and reconnects if position changed
async fn handle_config_change<F>(
    ctx: &OverlayContext<F>,
    overlay: &mut Option<Box<dyn OverlayBackend>>,
    new_overlay_config: crate::config::OverlayConfig,
    last_color: &mut Option<OverlayColor>,
) where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    let mut state = ctx.state().lock().await;
    state.update_config(new_overlay_config);
    let new_position = state.cached_position;
    tracing::info!("Overlay config updated");
    drop(state);

    if let Some(current_overlay) = overlay {
        if current_overlay.position() != new_position {
            tracing::info!("Overlay position changed, attempting immediate reconnection...");
            *overlay = None;

            *overlay = connect_and_initialize_backend(
                ctx,
                new_position,
                last_color,
                "after position change",
            )
            .await;

            return;
        }
    }

    let color = {
        let state = ctx.state().lock().await;
        state.current_color()
    };

    try_update_color_with_fallback(
        ctx,
        overlay,
        color,
        last_color,
        "during config update",
        false,
        false,
    )
    .await;
}

/// Handles activation state change - updates overlay color if state changed
async fn handle_activation_change<F>(
    ctx: &OverlayContext<F>,
    overlay: &mut Option<Box<dyn OverlayBackend>>,
    new_state: crate::activation::SystemState,
    last_color: &mut Option<OverlayColor>,
) where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    let mut state = ctx.state().lock().await;
    state.update_system_state(new_state);
    drop(state);

    let color = {
        let state = ctx.state().lock().await;
        state.current_color()
    };

    if Some(color) != *last_color {
        try_update_color_with_fallback(
            ctx,
            overlay,
            color,
            last_color,
            "during activation state change",
            true,
            true,
        )
        .await;
    }
}

/// Handles reconnection attempt with exponential backoff
async fn handle_reconnection_attempt<F>(
    ctx: &OverlayContext<F>,
    overlay: &mut Option<Box<dyn OverlayBackend>>,
    last_color: &mut Option<OverlayColor>,
) where
    F: Fn(OverlayPosition) -> Result<Box<dyn OverlayBackend>, wayland::WaylandError> + Send + Sync + 'static,
{
    if overlay.is_none() {
        let recon = ctx.reconnection().lock().await;
        if recon.should_retry() {
            drop(recon);

            let position = {
                let state = ctx.state().lock().await;
                state.cached_position
            };

            if let Some(new_backend) = connect_and_initialize_backend(
                ctx,
                position,
                last_color,
                "after reconnect",
            )
            .await
            {
                *overlay = Some(new_backend);
                tracing::info!("Overlay reconnected successfully");
            } else {
                let recon = ctx.reconnection().lock().await;
                let wait_time = recon.next_backoff();
                drop(recon);
                let mut state = ctx.state().lock().await;
                state.set_error(true);
                tracing::debug!("Will retry in {:?}", wait_time);
            }
        }
    }
}

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
    ///
    /// # Shutdown Behavior
    /// The spawned overlay task monitors both config and activation state changes.
    /// When either the `ConfigManager` or `ActivationManager` is dropped, their
    /// respective watch channels close, signaling the overlay task to exit gracefully.
    /// The task will:
    /// 1. Detect the channel closure
    /// 2. Break from the main event loop
    /// 3. Disconnect the overlay backend (if connected)
    /// 4. Exit cleanly without spinning in a hot loop
    ///
    /// This prevents resource exhaustion and ensures proper cleanup on shutdown.
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
        let mut config_health_rx = config_manager.health_subscribe();
        let mut activation_rx = activation_manager.subscribe();

        let backend_factory = Arc::new(backend_factory);

        let task_handle = tokio::spawn(async move {
            let ctx = OverlayContext::new(
                state_clone.clone(),
                reconnection_clone.clone(),
                backend_factory,
            );

            let mut last_color: Option<OverlayColor> = None;

            let position = {
                let state = ctx.state().lock().await;
                state.cached_position
            };

            let mut overlay = connect_and_initialize_backend(
                &ctx,
                position,
                &mut last_color,
                "after initial connection",
            )
            .await;

            let mut health_check_interval = tokio::time::interval(Duration::from_secs(2));
            let mut reconnection_interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = health_check_interval.tick() => {
                        handle_health_check(
                            &ctx,
                            &mut overlay,
                            &mut last_color,
                        )
                        .await;
                    }

                    config_change_result = config_rx.changed() => {
                        if let Err(_) = config_change_result {
                            tracing::info!("Config watcher closed, shutting down overlay task");
                            break;
                        }

                        let new_full_config = config_rx.borrow().clone();
                        let new_overlay_config = new_full_config.overlay.clone();

                        handle_config_change(
                            &ctx,
                            &mut overlay,
                            new_overlay_config,
                            &mut last_color,
                        )
                        .await;
                    }

                    activation_change_result = activation_rx.changed() => {
                        if let Err(_) = activation_change_result {
                            tracing::info!("Activation watcher closed, shutting down overlay task");
                            break;
                        }

                        let (new_state, _transition) = *activation_rx.borrow();

                        handle_activation_change(
                            &ctx,
                            &mut overlay,
                            new_state,
                            &mut last_color,
                        )
                        .await;
                    }

                    config_health_result = config_health_rx.changed() => {
                        if let Err(_) = config_health_result {
                            tracing::info!("Config health watcher closed, shutting down overlay task");
                            break;
                        }

                        let health = config_health_rx.borrow().clone();

                        match health {
                            WatcherHealth::Healthy => {
                                tracing::info!("Config watcher healthy");
                            }
                            WatcherHealth::Restarting { attempt } => {
                                tracing::warn!("Config watcher restarting (attempt {}), setting error state", attempt);
                                let mut state = ctx.state().lock().await;
                                state.set_error(true);
                                drop(state);

                                handle_health_check(
                                    &ctx,
                                    &mut overlay,
                                    &mut last_color,
                                )
                                .await;
                            }
                            WatcherHealth::Failed { ref reason } => {
                                tracing::error!("Config watcher failed: {}, setting error state", reason);
                                let mut state = ctx.state().lock().await;
                                state.set_error(true);
                                drop(state);

                                handle_health_check(
                                    &ctx,
                                    &mut overlay,
                                    &mut last_color,
                                )
                                .await;
                            }
                        }
                    }

                    _ = reconnection_interval.tick() => {
                        handle_reconnection_attempt(
                            &ctx,
                            &mut overlay,
                            &mut last_color,
                        )
                        .await;
                    }
                }
            }

            if let Some(mut backend) = overlay {
                backend.disconnect();
                tracing::debug!("Overlay backend disconnected during shutdown");
            }
            tracing::info!("Overlay task exited cleanly");
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

    /// Returns a snapshot of reconnection diagnostics
    ///
    /// Provides information about reconnection attempts, backoff state, and retry readiness.
    /// Useful for monitoring, debugging, and observability of the overlay connection health.
    pub async fn reconnection_status(&self) -> super::state::ReconnectionStatus {
        self.reconnection_state.lock().await.snapshot()
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}
