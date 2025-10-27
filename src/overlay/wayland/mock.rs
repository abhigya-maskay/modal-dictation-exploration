use super::{OverlayPosition, WaylandError};
use std::sync::{Arc, Mutex};

/// Shared state for MockWaylandProtocol
///
/// This allows external control of the mock even after it's been moved
/// into a Box<dyn WaylandProtocol>.
#[derive(Clone)]
pub struct MockProtocolHandle {
    surface_closed: Arc<Mutex<bool>>,
    inject_connect_error: Arc<Mutex<bool>>,
    inject_buffer_error: Arc<Mutex<bool>>,
    connect_count: Arc<Mutex<u32>>,
    disconnect_count: Arc<Mutex<u32>>,
    update_buffer_count: Arc<Mutex<u32>>,
    last_buffer_data: Arc<Mutex<Option<Vec<u8>>>>,
}

impl MockProtocolHandle {
    /// Simulates the compositor closing the surface
    pub fn simulate_surface_closed(&self) {
        *self.surface_closed.lock().unwrap() = true;
    }

    /// Clears the surface_closed flag
    pub fn clear_surface_closed(&self) {
        *self.surface_closed.lock().unwrap() = false;
    }

    /// Configures connect() to fail
    pub fn inject_connect_error(&self) {
        *self.inject_connect_error.lock().unwrap() = true;
    }

    /// Clears the connect error injection
    pub fn clear_connect_error(&self) {
        *self.inject_connect_error.lock().unwrap() = false;
    }

    /// Configures update_buffer() to fail
    pub fn inject_buffer_error(&self) {
        *self.inject_buffer_error.lock().unwrap() = true;
    }

    /// Clears the buffer error injection
    pub fn clear_buffer_error(&self) {
        *self.inject_buffer_error.lock().unwrap() = false;
    }

    /// Returns the number of successful connect() calls
    pub fn connect_count(&self) -> u32 {
        *self.connect_count.lock().unwrap()
    }

    /// Returns the number of disconnect() calls
    pub fn disconnect_count(&self) -> u32 {
        *self.disconnect_count.lock().unwrap()
    }

    /// Returns the number of successful update_buffer() calls
    pub fn update_buffer_count(&self) -> u32 {
        *self.update_buffer_count.lock().unwrap()
    }

    /// Returns a copy of the last buffer data
    pub fn last_buffer_data(&self) -> Option<Vec<u8>> {
        self.last_buffer_data.lock().unwrap().clone()
    }
}

/// Mock implementation of WaylandProtocol for testing
///
/// This mock allows full control over the Wayland protocol behavior for testing
/// reconnection logic, error handling, and state transitions without requiring
/// a running Wayland compositor.
pub struct MockWaylandProtocol {
    position: OverlayPosition,
    size: (u32, u32),
    connected: bool,
    handle: MockProtocolHandle,
}

impl MockWaylandProtocol {
    /// Creates a new mock Wayland protocol handler
    ///
    /// Returns the protocol and a handle for external control
    pub fn new(position: OverlayPosition, size: (u32, u32)) -> (Self, MockProtocolHandle) {
        let handle = MockProtocolHandle {
            surface_closed: Arc::new(Mutex::new(false)),
            inject_connect_error: Arc::new(Mutex::new(false)),
            inject_buffer_error: Arc::new(Mutex::new(false)),
            connect_count: Arc::new(Mutex::new(0)),
            disconnect_count: Arc::new(Mutex::new(0)),
            update_buffer_count: Arc::new(Mutex::new(0)),
            last_buffer_data: Arc::new(Mutex::new(None)),
        };

        let protocol = Self {
            position,
            size,
            connected: false,
            handle: handle.clone(),
        };

        (protocol, handle)
    }

}

impl super::protocol::WaylandProtocol for MockWaylandProtocol {
    fn connect(&mut self, position: OverlayPosition, size: (u32, u32)) -> Result<(), WaylandError> {
        if *self.handle.inject_connect_error.lock().unwrap() {
            tracing::debug!("MockWaylandProtocol: connect() failed (injected error)");
            return Err(WaylandError::ConnectionFailed);
        }

        self.position = position;
        self.size = size;
        self.connected = true;
        *self.handle.surface_closed.lock().unwrap() = false;
        *self.handle.connect_count.lock().unwrap() += 1;

        tracing::debug!(
            "MockWaylandProtocol: connected (position: {:?}, count: {})",
            position,
            *self.handle.connect_count.lock().unwrap()
        );
        Ok(())
    }

    fn is_surface_closed(&self) -> bool {
        *self.handle.surface_closed.lock().unwrap()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn update_buffer(&mut self, pixels: &[u8]) -> Result<(), WaylandError> {
        if !self.connected {
            return Err(WaylandError::CommitFailed);
        }

        if *self.handle.inject_buffer_error.lock().unwrap() {
            tracing::debug!("MockWaylandProtocol: update_buffer() failed (injected error)");
            return Err(WaylandError::BufferCreationFailed);
        }

        let expected_size = (self.size.0 * self.size.1 * 4) as usize;
        if pixels.len() != expected_size {
            return Err(WaylandError::BufferCreationFailed);
        }

        *self.handle.last_buffer_data.lock().unwrap() = Some(pixels.to_vec());
        *self.handle.update_buffer_count.lock().unwrap() += 1;

        tracing::debug!(
            "MockWaylandProtocol: buffer updated (count: {})",
            *self.handle.update_buffer_count.lock().unwrap()
        );
        Ok(())
    }

    fn disconnect(&mut self) {
        self.connected = false;
        *self.handle.disconnect_count.lock().unwrap() += 1;
        tracing::debug!(
            "MockWaylandProtocol: disconnected (count: {})",
            *self.handle.disconnect_count.lock().unwrap()
        );
    }

    fn position(&self) -> OverlayPosition {
        self.position
    }

    fn set_position(&mut self, position: OverlayPosition) {
        self.position = position;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::overlay::wayland::protocol::WaylandProtocol;

    #[test]
    fn test_mock_creation() {
        let (mock, _handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));
        assert!(!mock.is_connected());
        assert!(!mock.is_surface_closed());
        assert_eq!(mock.position(), OverlayPosition::TopRight);
    }

    #[test]
    fn test_mock_connect() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));
        assert_eq!(handle.connect_count(), 0);

        assert!(mock.connect(OverlayPosition::TopLeft, (32, 32)).is_ok());
        assert!(mock.is_connected());
        assert_eq!(handle.connect_count(), 1);
    }

    #[test]
    fn test_mock_connect_error_injection() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::BottomRight, (32, 32));
        handle.inject_connect_error();

        let result = mock.connect(OverlayPosition::BottomRight, (32, 32));
        assert!(result.is_err());
        assert!(!mock.is_connected());
        assert_eq!(handle.connect_count(), 0);

        handle.clear_connect_error();
        assert!(mock.connect(OverlayPosition::BottomRight, (32, 32)).is_ok());
        assert_eq!(handle.connect_count(), 1);
    }

    #[test]
    fn test_mock_surface_closed_simulation() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));
        mock.connect(OverlayPosition::TopRight, (32, 32)).unwrap();

        assert!(!mock.is_surface_closed());

        handle.simulate_surface_closed();
        assert!(mock.is_surface_closed());

        handle.clear_surface_closed();
        assert!(!mock.is_surface_closed());
    }

    #[test]
    fn test_mock_update_buffer() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));
        mock.connect(OverlayPosition::TopRight, (32, 32)).unwrap();

        let buffer_size = 32 * 32 * 4;
        let pixels = vec![0u8; buffer_size];

        assert!(mock.update_buffer(&pixels).is_ok());
        assert_eq!(handle.update_buffer_count(), 1);
        assert_eq!(handle.last_buffer_data().unwrap().len(), buffer_size);
    }

    #[test]
    fn test_mock_update_buffer_error_injection() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::BottomLeft, (32, 32));
        mock.connect(OverlayPosition::BottomLeft, (32, 32)).unwrap();
        handle.inject_buffer_error();

        let pixels = vec![0u8; 32 * 32 * 4];
        let result = mock.update_buffer(&pixels);

        assert!(result.is_err());
        assert_eq!(handle.update_buffer_count(), 0);

        handle.clear_buffer_error();
        assert!(mock.update_buffer(&pixels).is_ok());
        assert_eq!(handle.update_buffer_count(), 1);
    }

    #[test]
    fn test_mock_disconnect() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));
        mock.connect(OverlayPosition::TopLeft, (32, 32)).unwrap();

        assert!(mock.is_connected());
        assert_eq!(handle.disconnect_count(), 0);

        mock.disconnect();
        assert!(!mock.is_connected());
        assert_eq!(handle.disconnect_count(), 1);
    }

    #[test]
    fn test_mock_reconnection_clears_surface_closed() {
        let (mut mock, handle) = MockWaylandProtocol::new(OverlayPosition::BottomRight, (32, 32));
        mock.connect(OverlayPosition::BottomRight, (32, 32)).unwrap();
        handle.simulate_surface_closed();

        assert!(mock.is_surface_closed());

        mock.connect(OverlayPosition::BottomRight, (32, 32)).unwrap();
        assert!(!mock.is_surface_closed());
    }
}
