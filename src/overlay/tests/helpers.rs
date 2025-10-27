//! Test utilities for overlay manager integration tests
//!
//! Provides helper functions for creating test configurations
//! and mock backends with enhanced tracking capabilities.

use crate::activation::ActivationManager;
use crate::config::ConfigManager;
use crate::overlay::{
    OverlayBackend, OverlayColor, OverlayManager, OverlayPosition,
    MockOverlayBackend, wayland,
};
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;
use std::sync::Arc;
use tempfile::TempDir;

pub(crate) fn create_test_config_dir(config_content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("config.toml");
    std::fs::write(&config_path, config_content)
        .expect("Failed to write test config");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

pub(crate) fn create_default_test_config() -> (TempDir, PathBuf) {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    create_test_config_dir(config_content)
}

pub(crate) fn create_test_overlay_manager(
    config_mgr: &ConfigManager,
    activation_mgr: &Arc<ActivationManager>,
) -> OverlayManager {
    OverlayManager::new_with_factory(config_mgr, activation_mgr, |position| {
        MockOverlayBackend::new(position)
            .map(|backend| Box::new(backend) as Box<dyn OverlayBackend>)
    })
}

/// Wrapper around MockOverlayBackend that tracks all color updates externally
/// Useful for integration tests that need to verify backend received specific colors
pub(crate) struct TrackedMockBackend {
    inner: MockOverlayBackend,
    color_history: Arc<std::sync::Mutex<Vec<OverlayColor>>>,
}

impl TrackedMockBackend {
    pub(crate) fn new(
        position: OverlayPosition,
        color_history: Arc<std::sync::Mutex<Vec<OverlayColor>>>,
    ) -> Result<Self, wayland::WaylandError> {
        Ok(Self {
            inner: MockOverlayBackend::new(position)?,
            color_history,
        })
    }
}

impl OverlayBackend for TrackedMockBackend {
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<(), wayland::WaylandError>> + Send + '_>> {
        self.inner.connect()
    }

    fn update_color(&mut self, color: OverlayColor) -> Pin<Box<dyn Future<Output = Result<(), wayland::WaylandError>> + Send + '_>> {
        let history = self.color_history.clone();
        Box::pin(async move {
            let result = self.inner.update_color(color).await;
            if result.is_ok() {
                let mut colors = history.lock().unwrap();
                colors.push(color);
            }
            result
        })
    }

    fn disconnect(&mut self) {
        self.inner.disconnect()
    }

    fn position(&self) -> OverlayPosition {
        self.inner.position()
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}
