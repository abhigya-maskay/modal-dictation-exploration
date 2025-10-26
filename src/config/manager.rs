use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use notify::{Watcher, RecursiveMode, Event, EventKind};

use super::{Config, ConfigError};

/// Manages configuration with live reload capability
pub struct ConfigManager {
    /// Sender for broadcasting config updates
    tx: watch::Sender<Arc<Config>>,
    /// Receiver that can be cloned for subscribers
    rx: watch::Receiver<Arc<Config>>,
    /// Handle to the file watcher task
    watcher_task: JoinHandle<()>,
}

impl ConfigManager {
    /// Creates a new ConfigManager, loads the initial config, and starts watching for changes
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = Self::get_config_dir()?;
        Self::new_internal(config_dir)
    }

    #[cfg(test)]
    pub fn new_with_path(config_dir: PathBuf) -> Result<Self, ConfigError> {
        Self::new_internal(config_dir)
    }

    fn new_internal(config_dir: PathBuf) -> Result<Self, ConfigError> {
        let config_path = config_dir.join("config.toml");
        let initial_config = Config::load_from_path(config_path);

        tracing::info!("ConfigManager initialized with config");
        tracing::debug!("Initial config: {:?}", initial_config);

        let config_arc = Arc::new(initial_config);

        let (tx, rx) = watch::channel(config_arc.clone());

        let watcher_task = Self::spawn_watcher(tx.clone(), config_dir);

        Ok(Self {
            tx,
            rx,
            watcher_task,
        })
    }

    /// Returns a receiver that can be used to subscribe to config updates
    pub fn subscribe(&self) -> watch::Receiver<Arc<Config>> {
        self.rx.clone()
    }

    /// Returns the current config snapshot
    pub fn current(&self) -> Arc<Config> {
        self.rx.borrow().clone()
    }

    /// Spawns the file watcher task that monitors config file changes
    fn spawn_watcher(tx: watch::Sender<Arc<Config>>, config_dir: PathBuf) -> JoinHandle<()> {
        tokio::spawn(async move {
            if let Err(e) = Self::watch_config_file(tx, config_dir).await {
                tracing::error!("Config watcher task failed: {}", e);
            }
        })
    }

    /// Main watcher loop that monitors the config directory for changes
    async fn watch_config_file(tx: watch::Sender<Arc<Config>>, config_dir: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config_path = config_dir.join("config.toml");

        if !config_dir.exists() {
            tracing::info!("Config directory does not exist, creating: {}", config_dir.display());
            tokio::fs::create_dir_all(&config_dir).await?;
        }

        tracing::info!("Starting config file watcher for: {}", config_path.display());

        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    tracing::debug!("File system event: {:?}", event);
                    let _ = event_tx.blocking_send(event);
                }
                Err(e) => {
                    tracing::warn!("File watcher error: {}", e);
                }
            }
        })?;

        watcher.watch(&config_dir, RecursiveMode::NonRecursive)?;
        tracing::debug!("Watching directory: {}", config_dir.display());

        let mut debounce_timer: Option<tokio::time::Instant> = None;
        let debounce_duration = Duration::from_millis(500);

        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    let is_config_event = event.paths.iter().any(|p| {
                        p.file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| name == "config.toml")
                            .unwrap_or(false)
                    });

                    if !is_config_event {
                        continue;
                    }

                    let should_reload = matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_)
                    );

                    if should_reload {
                        tracing::debug!("Config file change detected, starting debounce timer");
                        debounce_timer = Some(tokio::time::Instant::now() + debounce_duration);
                    }
                }

                _ = async {
                    if let Some(deadline) = debounce_timer {
                        tokio::time::sleep_until(deadline).await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                }, if debounce_timer.is_some() => {
                    tracing::debug!("Debounce period elapsed, reloading config");
                    debounce_timer = None;

                    Self::reload_config(&tx, &config_path).await;
                }
            }
        }
    }

    /// Attempts to reload the config file and broadcast updates
    async fn reload_config(tx: &watch::Sender<Arc<Config>>, config_path: &PathBuf) {
        tracing::info!("Reloading config from: {}", config_path.display());

        match tokio::fs::read_to_string(config_path).await {
            Ok(contents) => {
                match toml::from_str::<Config>(&contents) {
                    Ok(new_config) => {
                        let config_arc = Arc::new(new_config);
                        if tx.send(config_arc).is_err() {
                            tracing::warn!("No active config subscribers");
                        } else {
                            tracing::info!("Config reloaded successfully and broadcast to subscribers");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse config file: {}, keeping last valid config", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read config file: {}, keeping last valid config", e);
            }
        }
    }

    /// Gets the config directory path
    fn get_config_dir() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir()
            .ok_or(ConfigError::DirectoryNotFound)?
            .join("phonesc");

        Ok(config_dir)
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        self.watcher_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_loads_initial_config() {
        let manager = ConfigManager::new().unwrap();
        let config = manager.current();

        assert!(config.auto_sleep_timeout_secs > 0);
        assert!(config.command_pause_threshold_ms > 0);
        assert!(config.dictation_pause_threshold_ms > 0);
    }

    #[tokio::test]
    async fn test_subscribe_receives_current_config() {
        let manager = ConfigManager::new().unwrap();
        let subscriber = manager.subscribe();

        let config = subscriber.borrow().clone();
        assert!(config.auto_sleep_timeout_secs > 0);

        assert!(!config.overlay.position.is_empty());
        assert!(!config.dictation_service.host.is_empty());
        assert!(config.dictation_service.port > 0);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = ConfigManager::new().unwrap();
        let sub1 = manager.subscribe();
        let sub2 = manager.subscribe();

        let config1 = sub1.borrow().clone();
        let config2 = sub2.borrow().clone();

        assert_eq!(config1.auto_sleep_timeout_secs, config2.auto_sleep_timeout_secs);
        assert_eq!(config1.command_pause_threshold_ms, config2.command_pause_threshold_ms);
        assert_eq!(config1.overlay.position, config2.overlay.position);
    }

    #[tokio::test]
    async fn test_current_returns_same_config() {
        let manager = ConfigManager::new().unwrap();
        let config1 = manager.current();
        let config2 = manager.current();

        assert_eq!(config1.auto_sleep_timeout_secs, config2.auto_sleep_timeout_secs);
    }

    #[tokio::test]
    async fn test_subscriber_can_detect_changes() {
        let manager = ConfigManager::new().unwrap();
        let subscriber = manager.subscribe();

        let initial_config = subscriber.borrow().clone();
        assert!(initial_config.auto_sleep_timeout_secs > 0);

        assert!(!subscriber.has_changed().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_config_manager_initialization_doesnt_panic() {
        let result = ConfigManager::new();
        assert!(result.is_ok(), "ConfigManager should initialize even without config file");
    }

    #[tokio::test]
    async fn test_get_config_dir_returns_path() {
        let result = ConfigManager::get_config_dir();
        if let Ok(path) = result {
            assert!(path.ends_with("phonesc"));
        }
    }

    #[tokio::test]
    async fn test_config_reload_on_file_change() {
        use tempfile::TempDir;
        use tokio::time::{timeout, Duration};

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, r#"
auto_sleep_timeout_secs = 300
command_pause_threshold_ms = 700
dictation_pause_threshold_ms = 900
        "#).unwrap();

        let manager = ConfigManager::new_with_path(temp_dir.path().to_path_buf()).unwrap();
        let mut subscriber = manager.subscribe();

        let initial_config = subscriber.borrow().clone();
        assert_eq!(initial_config.auto_sleep_timeout_secs, 300);

        tokio::time::sleep(Duration::from_millis(100)).await;

        std::fs::write(&config_path, r#"
auto_sleep_timeout_secs = 600
command_pause_threshold_ms = 800
dictation_pause_threshold_ms = 1000
        "#).unwrap();

        let changed = timeout(Duration::from_secs(2), subscriber.changed()).await;
        assert!(changed.is_ok(), "Timeout waiting for config change");
        assert!(changed.unwrap().is_ok(), "Config change notification failed");

        let updated_config = subscriber.borrow().clone();
        assert_eq!(updated_config.auto_sleep_timeout_secs, 600);
        assert_eq!(updated_config.command_pause_threshold_ms, 800);
    }

    #[tokio::test]
    async fn test_invalid_config_preserves_last_good() {
        use tempfile::TempDir;
        use tokio::time::{timeout, Duration};

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, r#"
auto_sleep_timeout_secs = 300
command_pause_threshold_ms = 700
        "#).unwrap();

        let manager = ConfigManager::new_with_path(temp_dir.path().to_path_buf()).unwrap();
        let mut subscriber = manager.subscribe();

        let initial_config = subscriber.borrow().clone();
        assert_eq!(initial_config.auto_sleep_timeout_secs, 300);

        tokio::time::sleep(Duration::from_millis(100)).await;

        std::fs::write(&config_path, "invalid { toml syntax").unwrap();

        let result = timeout(Duration::from_millis(800), subscriber.changed()).await;
        assert!(result.is_err(), "Should NOT receive notification for invalid config - timeout expected");

        let config_after_invalid = subscriber.borrow().clone();
        assert_eq!(config_after_invalid.auto_sleep_timeout_secs, 300,
            "Config should remain unchanged after invalid write");
    }

    #[tokio::test]
    async fn test_debouncing_multiple_rapid_writes() {
        use tempfile::TempDir;
        use tokio::time::Duration;
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc as StdArc;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, r#"
auto_sleep_timeout_secs = 100
        "#).unwrap();

        let manager = ConfigManager::new_with_path(temp_dir.path().to_path_buf()).unwrap();
        let mut subscriber = manager.subscribe();

        let change_count = StdArc::new(AtomicU32::new(0));
        let change_count_clone = change_count.clone();

        tokio::spawn(async move {
            while subscriber.changed().await.is_ok() {
                change_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        for i in 1..=5 {
            std::fs::write(&config_path, format!("auto_sleep_timeout_secs = {}", 100 + i * 10)).unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        tokio::time::sleep(Duration::from_millis(800)).await;

        let count = change_count.load(Ordering::SeqCst);
        assert!(count <= 2, "Expected at most 2 reloads due to debouncing, got {}", count);
    }

    #[tokio::test]
    async fn test_config_directory_creation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("nonexistent");

        assert!(!config_dir.exists(), "Config directory should not exist initially");

        let manager = ConfigManager::new_with_path(config_dir.clone()).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert!(config_dir.exists(), "Config directory should be created by watcher");

        let config = manager.current();
        assert!(config.auto_sleep_timeout_secs > 0);
    }

    #[tokio::test]
    async fn test_atomic_editor_writes() {
        use tempfile::TempDir;
        use tokio::time::{timeout, Duration};

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        std::fs::write(&config_path, r#"
auto_sleep_timeout_secs = 300
        "#).unwrap();

        let manager = ConfigManager::new_with_path(temp_dir.path().to_path_buf()).unwrap();
        let mut subscriber = manager.subscribe();

        tokio::time::sleep(Duration::from_millis(100)).await;

        let temp_file = temp_dir.path().join(".config.toml.tmp");
        std::fs::write(&temp_file, r#"
auto_sleep_timeout_secs = 999
        "#).unwrap();
        std::fs::rename(&temp_file, &config_path).unwrap();

        let changed = timeout(Duration::from_secs(2), subscriber.changed()).await;
        assert!(changed.is_ok(), "Timeout waiting for config change after atomic rename");

        if changed.unwrap().is_ok() {
            let updated_config = subscriber.borrow().clone();
            assert_eq!(updated_config.auto_sleep_timeout_secs, 999);
        }
    }
}
