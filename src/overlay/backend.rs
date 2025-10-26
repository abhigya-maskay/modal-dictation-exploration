use crate::overlay::renderer::OverlayColor;
use crate::overlay::wayland::{OverlayPosition, WaylandError};
use std::future::Future;
use std::pin::Pin;

/// Trait for overlay backend implementations
pub trait OverlayBackend: Send + Sync {
    /// Attempts to connect to the overlay backend
    /// Returns a Send future to allow use in tokio::spawn
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>>;

    /// Updates the overlay color
    /// Returns a Send future to allow use in tokio::spawn
    fn update_color(&mut self, color: OverlayColor) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>>;

    /// Disconnects from the overlay backend
    fn disconnect(&mut self);

    /// Returns the current position
    fn position(&self) -> OverlayPosition;

    /// Returns whether the backend is connected
    fn is_connected(&self) -> bool;
}

/// Mock overlay backend for testing
/// Always succeeds, never fails, useful for headless systems
pub struct MockOverlayBackend {
    position: OverlayPosition,
    connected: bool,
    last_color: Option<OverlayColor>,
}

impl MockOverlayBackend {
    /// Creates a new mock overlay backend
    pub fn new(position: OverlayPosition) -> Result<Self, WaylandError> {
        Ok(Self {
            position,
            connected: false,
            last_color: None,
        })
    }
}

/// Mock overlay backend that can be configured to fail in specific ways
/// Useful for testing error handling and reconnection logic
pub struct FailingMockBackend {
    position: OverlayPosition,
    connected: bool,
    last_color: Option<OverlayColor>,
    /// Controls whether connect() should fail (if Some, fail N times then succeed)
    connect_fail_count: std::sync::Arc<std::sync::Mutex<Option<u32>>>,
    /// Controls whether update_color() should fail (if Some, fail N times then succeed)
    update_color_fail_count: std::sync::Arc<std::sync::Mutex<Option<u32>>>,
    /// Tracks number of successful connects for testing
    connect_attempts: std::sync::Arc<std::sync::Mutex<u32>>,
    /// Tracks number of color update attempts
    update_attempts: std::sync::Arc<std::sync::Mutex<u32>>,
}

impl FailingMockBackend {
    pub fn new(position: OverlayPosition) -> Result<Self, WaylandError> {
        Ok(Self {
            position,
            connected: false,
            last_color: None,
            connect_fail_count: std::sync::Arc::new(std::sync::Mutex::new(None)),
            update_color_fail_count: std::sync::Arc::new(std::sync::Mutex::new(None)),
            connect_attempts: std::sync::Arc::new(std::sync::Mutex::new(0)),
            update_attempts: std::sync::Arc::new(std::sync::Mutex::new(0)),
        })
    }

    /// Configure connect() to fail N times before succeeding
    pub fn fail_connect_n_times(self, n: u32) -> Self {
        *self.connect_fail_count.lock().unwrap() = Some(n);
        self
    }

    /// Configure update_color() to fail N times before succeeding
    pub fn fail_update_color_n_times(self, n: u32) -> Self {
        *self.update_color_fail_count.lock().unwrap() = Some(n);
        self
    }

    /// Get the number of connect attempts
    pub fn connect_attempt_count(&self) -> u32 {
        *self.connect_attempts.lock().unwrap()
    }

    /// Get the number of update attempts
    pub fn update_attempt_count(&self) -> u32 {
        *self.update_attempts.lock().unwrap()
    }

    /// Get the last color that was successfully updated
    pub fn last_color(&self) -> Option<OverlayColor> {
        self.last_color
    }
}

impl OverlayBackend for MockOverlayBackend {
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>> {
        Box::pin(async move {
            tracing::debug!("MockOverlayBackend connecting (position: {:?})", self.position);
            self.connected = true;
            Ok(())
        })
    }

    fn update_color(&mut self, color: OverlayColor) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>> {
        Box::pin(async move {
            if !self.connected {
                self.connected = true;
            }
            tracing::debug!(
                "MockOverlayBackend color updated (position: {:?}, color: {:?})",
                self.position,
                color
            );
            self.last_color = Some(color);
            Ok(())
        })
    }

    fn disconnect(&mut self) {
        self.connected = false;
        tracing::debug!("MockOverlayBackend disconnected");
    }

    fn position(&self) -> OverlayPosition {
        self.position
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl OverlayBackend for FailingMockBackend {
    fn connect(&mut self) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>> {
        let mut fail_count = self.connect_fail_count.lock().unwrap();
        let should_fail = match fail_count.as_mut() {
            Some(count) if *count > 0 => {
                *count -= 1;
                true
            }
            _ => false,
        };
        drop(fail_count);

        let mut attempts = self.connect_attempts.lock().unwrap();
        *attempts += 1;
        drop(attempts);

        let position = self.position;
        Box::pin(async move {
            tracing::debug!("FailingMockBackend connecting (position: {:?})", position);
            if should_fail {
                Err(WaylandError::ConnectionFailed)
            } else {
                Ok(())
            }
        })
    }

    fn update_color(&mut self, color: OverlayColor) -> Pin<Box<dyn Future<Output = Result<(), WaylandError>> + Send + '_>> {
        let mut fail_count = self.update_color_fail_count.lock().unwrap();
        let should_fail = match fail_count.as_mut() {
            Some(count) if *count > 0 => {
                *count -= 1;
                true
            }
            _ => false,
        };
        drop(fail_count);

        let mut attempts = self.update_attempts.lock().unwrap();
        *attempts += 1;
        drop(attempts);

        let position = self.position;
        Box::pin(async move {
            tracing::debug!(
                "FailingMockBackend color update (position: {:?}, color: {:?})",
                position,
                color
            );
            if should_fail {
                Err(WaylandError::SurfaceCreationFailed)
            } else {
                Ok(())
            }
        })
    }

    fn disconnect(&mut self) {
        tracing::debug!("FailingMockBackend disconnected");
    }

    fn position(&self) -> OverlayPosition {
        self.position
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_overlay_creation() {
        let backend =
            MockOverlayBackend::new(OverlayPosition::TopRight).expect("Failed to create backend");
        assert_eq!(backend.position(), OverlayPosition::TopRight);
        assert!(!backend.is_connected());
    }

    #[tokio::test]
    async fn test_mock_overlay_connect() {
        let mut backend =
            MockOverlayBackend::new(OverlayPosition::TopLeft).expect("Failed to create backend");
        assert!(backend.connect().await.is_ok());
        assert!(backend.is_connected());
    }

    #[tokio::test]
    async fn test_mock_overlay_color_update() {
        let mut backend =
            MockOverlayBackend::new(OverlayPosition::BottomRight).expect("Failed to create backend");
        let color = OverlayColor::opaque(255, 0, 0);
        assert!(backend.update_color(color).await.is_ok());
        assert!(backend.is_connected());
    }

    #[tokio::test]
    async fn test_mock_overlay_disconnect() {
        let mut backend =
            MockOverlayBackend::new(OverlayPosition::BottomLeft).expect("Failed to connect");
        backend.connect().await.expect("Failed to connect");
        assert!(backend.is_connected());
        backend.disconnect();
        assert!(!backend.is_connected());
    }

    #[tokio::test]
    async fn test_failing_mock_connect_failure() {
        let mut backend =
            FailingMockBackend::new(OverlayPosition::TopRight).expect("Failed to create backend");
        backend = backend.fail_connect_n_times(1);

        let result = backend.connect().await;
        assert!(result.is_err());
        assert_eq!(backend.connect_attempt_count(), 1);
    }

    #[tokio::test]
    async fn test_failing_mock_connect_success_after_failures() {
        let mut backend =
            FailingMockBackend::new(OverlayPosition::TopRight).expect("Failed to create backend");
        backend = backend.fail_connect_n_times(2);

        assert!(backend.connect().await.is_err());
        assert_eq!(backend.connect_attempt_count(), 1);

        assert!(backend.connect().await.is_err());
        assert_eq!(backend.connect_attempt_count(), 2);

        assert!(backend.connect().await.is_ok());
        assert_eq!(backend.connect_attempt_count(), 3);
    }

    #[tokio::test]
    async fn test_failing_mock_update_failure() {
        let mut backend =
            FailingMockBackend::new(OverlayPosition::BottomLeft).expect("Failed to create backend");
        backend = backend.fail_update_color_n_times(1);

        let color = OverlayColor::opaque(255, 0, 0);
        let result = backend.update_color(color).await;
        assert!(result.is_err());
        assert_eq!(backend.update_attempt_count(), 1);
    }

    #[tokio::test]
    async fn test_failing_mock_update_success_after_failures() {
        let mut backend =
            FailingMockBackend::new(OverlayPosition::BottomLeft).expect("Failed to create backend");
        backend = backend.fail_update_color_n_times(1);

        let color = OverlayColor::opaque(255, 0, 0);

        assert!(backend.update_color(color).await.is_err());
        assert_eq!(backend.update_attempt_count(), 1);

        assert!(backend.update_color(color).await.is_ok());
        assert_eq!(backend.update_attempt_count(), 2);
    }
}
