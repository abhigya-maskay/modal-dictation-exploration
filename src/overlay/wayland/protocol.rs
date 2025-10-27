use super::{OverlayPosition, WaylandError};

/// Trait abstracting Wayland protocol operations for testability
///
/// This trait encapsulates all Wayland-specific operations (connection management,
/// surface lifecycle, buffer handling) to enable testing reconnection logic without
/// requiring a running Wayland compositor.
///
/// Implementations:
/// - `ProductionWaylandProtocol`: Real Wayland operations using smithay-client-toolkit
/// - `MockWaylandProtocol`: Test implementation that simulates errors and state transitions
pub trait WaylandProtocol: Send + Sync {
    /// Attempts to establish Wayland connection and create configured layer surface
    ///
    /// # Arguments
    /// * `position` - Screen corner position for the overlay
    /// * `size` - Overlay dimensions (width, height) in pixels
    ///
    /// # Returns
    /// * `Ok(())` if connection established and surface configured
    /// * `Err(WaylandError)` if connection fails, globals missing, or surface creation fails
    fn connect(&mut self, position: OverlayPosition, size: (u32, u32)) -> Result<(), WaylandError>;

    /// Returns whether the compositor closed the layer surface
    ///
    /// This is the key signal for reconnection: when true, the overlay should
    /// disconnect and attempt to reconnect (compositor restart scenario).
    fn is_surface_closed(&self) -> bool;

    /// Returns whether currently connected to Wayland compositor
    fn is_connected(&self) -> bool;

    /// Updates the surface with new pixel data
    ///
    /// Creates a shared memory buffer from the provided pixels and attaches it to the surface.
    /// Processes Wayland events after committing to detect surface_closed events.
    ///
    /// # Arguments
    /// * `pixels` - BGRA8888 pixel data (width * height * 4 bytes)
    ///
    /// # Returns
    /// * `Ok(())` if buffer created, attached, and committed successfully
    /// * `Err(WaylandError)` if buffer creation fails or commit fails
    fn update_buffer(&mut self, pixels: &[u8]) -> Result<(), WaylandError>;

    /// Disconnects from Wayland compositor and cleans up resources
    fn disconnect(&mut self);

    /// Returns the current overlay position
    fn position(&self) -> OverlayPosition;

    /// Sets the overlay position (for reconnection after config change)
    fn set_position(&mut self, position: OverlayPosition);
}
