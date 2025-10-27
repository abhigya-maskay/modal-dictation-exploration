use super::{OverlayPosition, WaylandError};
use super::protocol::WaylandProtocol;
use super::production::ProductionWaylandProtocol;
use crate::overlay::renderer::OverlayColor;

/// Manages a Wayland surface for the overlay indicator
///
/// This implementation uses the wlr-layer-shell protocol backed by smithay-client-toolkit's
/// layer shell abstractions for proper overlay positioning.
/// It creates a small 32x32px layer surface anchored to a screen corner.
///
/// The overlay now uses a pluggable protocol backend (WaylandProtocol trait) to enable:
/// - Production use with real Wayland compositor (ProductionWaylandProtocol)
/// - Testing without a compositor (MockWaylandProtocol)
pub struct WaylandOverlay {
    position: OverlayPosition,
    size: (u32, u32),
    protocol: Box<dyn WaylandProtocol>,
    surface_closed: bool,
}

impl WaylandOverlay {
    /// Creates a new Wayland overlay manager with production protocol
    pub fn new(position: OverlayPosition) -> Result<Self, WaylandError> {
        Self::new_with_protocol(position, Box::new(ProductionWaylandProtocol::new(position, (32, 32))))
    }

    /// Creates a new Wayland overlay manager with a custom protocol implementation
    ///
    /// This constructor allows injecting test implementations for testing reconnection
    /// logic without requiring a running Wayland compositor.
    pub fn new_with_protocol(
        position: OverlayPosition,
        protocol: Box<dyn WaylandProtocol>,
    ) -> Result<Self, WaylandError> {
        tracing::debug!("WaylandOverlay initialized for position: {:?}", position);

        Ok(Self {
            position,
            size: (32, 32),
            protocol,
            surface_closed: false,
        })
    }

    /// Attempts to connect to the Wayland compositor
    pub fn connect(&mut self) -> Result<(), WaylandError> {
        tracing::info!(
            "WaylandOverlay attempting connection (position: {:?})",
            self.position
        );

        self.protocol.connect(self.position, self.size)?;
        self.surface_closed = false;

        tracing::info!("Wayland overlay connected successfully and configured");
        Ok(())
    }

    /// Updates the overlay color by attaching a new buffer
    ///
    /// This method implements the key reconnection logic:
    /// 1. Check if surface was closed by compositor (surface_closed flag)
    /// 2. If closed, disconnect and reconnect (compositor restart scenario)
    /// 3. Auto-connect if not currently connected
    /// 4. Render and update the buffer
    /// 5. Check for surface_closed after update (via protocol.is_surface_closed())
    pub fn update_color(&mut self, color: OverlayColor) -> Result<(), WaylandError> {
        if self.surface_closed || self.protocol.is_surface_closed() {
            tracing::warn!("Layer surface was closed by compositor, reconnecting...");
            self.surface_closed = false;
            self.disconnect();
            self.connect()?;
        }

        if !self.protocol.is_connected() {
            self.connect()?;
        }

        let rgba_pixel_data = crate::overlay::renderer::render_circle(color);
        let bgra_pixel_data = crate::overlay::renderer::rgba_to_bgra(&rgba_pixel_data);

        self.protocol.update_buffer(&bgra_pixel_data)?;

        if self.protocol.is_surface_closed() {
            self.surface_closed = true;
        }

        tracing::debug!(
            "WaylandOverlay color updated (position: {:?})",
            self.position
        );
        Ok(())
    }

    /// Disconnects from the Wayland compositor
    pub fn disconnect(&mut self) {
        self.protocol.disconnect();
        tracing::debug!("WaylandOverlay disconnected");
    }

    /// Returns the current position
    pub fn position(&self) -> OverlayPosition {
        self.protocol.position()
    }

    /// Returns the overlay size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Returns whether the overlay is currently connected
    pub fn is_connected(&self) -> bool {
        self.protocol.is_connected()
    }
}

impl crate::overlay::backend::OverlayBackend for WaylandOverlay {
    fn connect(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WaylandError>> + Send + '_>> {
        Box::pin(async {
            tokio::task::block_in_place(|| WaylandOverlay::connect(self))
        })
    }

    fn update_color(&mut self, color: OverlayColor) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), WaylandError>> + Send + '_>> {
        Box::pin(async move {
            tokio::task::block_in_place(|| WaylandOverlay::update_color(self, color))
        })
    }

    fn disconnect(&mut self) {
        WaylandOverlay::disconnect(self)
    }

    fn position(&self) -> OverlayPosition {
        WaylandOverlay::position(self)
    }

    fn is_connected(&self) -> bool {
        WaylandOverlay::is_connected(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: WaylandOverlay instantiation
    ///
    /// Verifies that creating a WaylandOverlay with ProductionWaylandProtocol
    /// doesn't panic and initializes with correct defaults.
    #[test]
    fn test_wayland_overlay_creation() {
        let overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        assert_eq!(overlay.position(), OverlayPosition::TopRight);
        assert_eq!(overlay.size(), (32, 32));
        assert!(!overlay.is_connected(), "Should not be connected on creation");
    }

    /// Test: Production connect error handling without compositor
    ///
    /// Verifies that WaylandOverlay with ProductionWaylandProtocol handles
    /// connection failures gracefully when no compositor is available.
    /// This tests the error path through the real Wayland connection code.
    #[test]
    fn test_production_connect_fails_without_compositor() {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            eprintln!("test_production_connect_fails_without_compositor: Skipping (WAYLAND_DISPLAY is set)");
            return;
        }

        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let result = overlay.connect();

        match result {
            Err(WaylandError::ConnectionFailed) |
            Err(WaylandError::MissingGlobals) |
            Err(WaylandError::LayerShellUnavailable) |
            Err(WaylandError::SurfaceCreationFailed) => {
                assert!(!overlay.is_connected());
            }
            Ok(()) => {
                panic!("Expected connection to fail without compositor, but it succeeded");
            }
            Err(e) => {
                panic!("Unexpected error type: {}", e);
            }
        }
    }

    /// Test: Production auto-connect error handling in update_color
    ///
    /// Verifies that update_color's auto-connect feature fails gracefully
    /// when no compositor is available. Tests the error path through
    /// ProductionWaylandProtocol.
    #[test]
    fn test_production_auto_connect_fails_without_compositor() {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            eprintln!("test_production_auto_connect_fails_without_compositor: Skipping (WAYLAND_DISPLAY is set)");
            return;
        }

        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let color = OverlayColor::opaque(0, 255, 0);
        let result = overlay.update_color(color);

        match result {
            Err(WaylandError::ConnectionFailed) |
            Err(WaylandError::MissingGlobals) |
            Err(WaylandError::LayerShellUnavailable) |
            Err(WaylandError::SurfaceCreationFailed) => {
                assert!(!overlay.is_connected());
            }
            Ok(()) => {
                panic!("Expected update_color to fail without compositor, but it succeeded");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    /// Integration test: Successful production connection with compositor
    ///
    /// This test REQUIRES a real Wayland compositor (WAYLAND_DISPLAY set).
    /// It verifies the complete production code path including:
    /// - smithay-client-toolkit connection and global binding
    /// - wlr-layer-shell surface creation
    /// - Configure event handling
    /// - Surface state management
    ///
    /// This complements the smoke tests which verify error paths.
    #[test]
    fn test_production_connection_succeeds_with_compositor() {
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            eprintln!("test_production_connection_succeeds_with_compositor: Skipping (WAYLAND_DISPLAY not set)");
            return;
        }

        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let result = overlay.connect();
        assert!(result.is_ok(), "Connection should succeed with Wayland display");
        assert!(overlay.is_connected());
    }

    /// Integration test: Production color update with compositor
    ///
    /// This test REQUIRES a real Wayland compositor (WAYLAND_DISPLAY set).
    /// It verifies the complete buffer update path including:
    /// - Buffer creation via smithay SlotPool
    /// - Buffer attachment to surface
    /// - Surface commit
    /// - Event processing after commit
    ///
    /// This tests what the smoke tests cannot: actual Wayland protocol interaction.
    #[test]
    fn test_production_color_update_succeeds_with_compositor() {
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            eprintln!("test_production_color_update_succeeds_with_compositor: Skipping (WAYLAND_DISPLAY not set)");
            return;
        }

        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        overlay.connect().expect("Failed to connect");

        let color = OverlayColor::opaque(0, 255, 0);
        let result = overlay.update_color(color);
        assert!(result.is_ok(), "Color update should succeed with connected overlay");
    }
}
