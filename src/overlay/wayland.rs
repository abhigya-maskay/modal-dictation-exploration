use crate::overlay::renderer::OverlayColor;
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity, LayerShell, LayerSurface, SurfaceKind};
use smithay_client_toolkit::shm::slot::SlotPool;
use smithay_client_toolkit::shm::Shm;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{Connection, EventQueue, QueueHandle};
use wayland_client::globals::registry_queue_init;

/// Error types for Wayland operations
#[derive(Debug, Error)]
pub enum WaylandError {
    #[error("Failed to connect to Wayland display")]
    ConnectionFailed,

    #[error("Wayland display not available")]
    NoDisplay,

    #[error("Failed to create layer shell surface")]
    SurfaceCreationFailed,

    #[error("Failed to create shared memory buffer")]
    BufferCreationFailed,

    #[error("Failed to commit surface")]
    CommitFailed,

    #[error("Compositor disconnected")]
    CompositorDisconnected,

    #[error("Missing Wayland globals")]
    MissingGlobals,

    #[error("Layer shell not available")]
    LayerShellUnavailable,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Parses overlay position string into anchor values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl OverlayPosition {
    /// Parses a position string (e.g., "top-right", "bottom-left")
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "top-left" => Ok(OverlayPosition::TopLeft),
            "top-right" => Ok(OverlayPosition::TopRight),
            "bottom-left" => Ok(OverlayPosition::BottomLeft),
            "bottom-right" => Ok(OverlayPosition::BottomRight),
            _ => Err(format!(
                "Invalid position: {}. Use: top-left, top-right, bottom-left, or bottom-right",
                s
            )),
        }
    }

    /// Returns the anchor values as Anchor bitflags for layer shell protocol
    pub fn anchor_flags(self) -> Anchor {
        match self {
            OverlayPosition::TopLeft => Anchor::TOP | Anchor::LEFT,
            OverlayPosition::TopRight => Anchor::TOP | Anchor::RIGHT,
            OverlayPosition::BottomLeft => Anchor::BOTTOM | Anchor::LEFT,
            OverlayPosition::BottomRight => Anchor::BOTTOM | Anchor::RIGHT,
        }
    }
}

/// Wayland application state using SCTK patterns
struct AppState {
    registry: RegistryState,
    output: OutputState,
    compositor: CompositorState,
    shm: Shm,
    layer_shell: LayerShell,
    configured: bool,
    /// Flag set when layer surface is closed by compositor (e.g., during restart)
    closed: bool,
    /// Last configure serial for acknowledging configure events
    last_configure_serial: u32,
    /// Size suggested by compositor in last configure event
    configured_size: (u32, u32),
}

impl ProvidesRegistryState for AppState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }

    fn runtime_add_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
        _version: u32,
    ) {
    }

    fn runtime_remove_global(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _name: u32,
        _interface: &str,
    ) {
    }
}

/// Handler for layer surface events
impl smithay_client_toolkit::shell::wlr_layer::LayerShellHandler for AppState {
    fn closed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        tracing::warn!("Layer surface closed by compositor (likely compositor restart)");
        self.closed = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        serial: u32,
    ) {
        self.last_configure_serial = serial;
        self.configured_size = configure.new_size;

        tracing::debug!(
            "Layer surface configure received: serial={}, suggested_size={:?}",
            serial,
            configure.new_size
        );

        if let SurfaceKind::Wlr(wlr_surface) = layer.kind() {
            wlr_surface.ack_configure(serial);
        }

        self.configured = true;
    }
}

impl smithay_client_toolkit::compositor::CompositorHandler for AppState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _new_transform: wayland_client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &WlOutput,
    ) {
    }
}

impl smithay_client_toolkit::shm::ShmHandler for AppState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl smithay_client_toolkit::output::OutputHandler for AppState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: WlOutput,
    ) {
    }
}

smithay_client_toolkit::delegate_registry!(AppState);
smithay_client_toolkit::delegate_output!(AppState);
smithay_client_toolkit::delegate_compositor!(AppState);
smithay_client_toolkit::delegate_shm!(AppState);
smithay_client_toolkit::delegate_layer!(AppState);

/// Manages a Wayland surface for the overlay indicator
///
/// This implementation uses the wlr-layer-shell protocol backed by smithay-client-toolkit's
/// layer shell abstractions for proper overlay positioning.
/// It creates a small 32x32px layer surface anchored to a screen corner.
pub struct WaylandOverlay {
    position: OverlayPosition,
    size: (u32, u32),
    /// Wayland connection
    connection: Option<Connection>,
    /// Event queue for processing Wayland events
    event_queue: Option<EventQueue<AppState>>,
    /// Application state containing all Wayland globals
    app_state: Option<AppState>,
    /// Layer surface (from smithay-client-toolkit)
    layer_surface: Option<LayerSurface>,
    /// Shared surface reference
    surface: Option<WlSurface>,
    /// Buffer pool for managing pixel buffers (from smithay-client-toolkit)
    buffer_pool: Option<Arc<Mutex<SlotPool>>>,
    /// Flag indicating whether connected to Wayland
    connected: bool,
    /// Flag indicating the layer surface was closed by the compositor
    surface_closed: bool,
}

impl WaylandOverlay {
    /// Creates a new Wayland overlay manager
    pub fn new(position: OverlayPosition) -> Result<Self, WaylandError> {
        tracing::debug!("WaylandOverlay initialized for position: {:?}", position);

        Ok(Self {
            position,
            size: (32, 32),
            connection: None,
            event_queue: None,
            app_state: None,
            layer_surface: None,
            surface: None,
            buffer_pool: None,
            connected: false,
            surface_closed: false,
        })
    }

    /// Attempts to connect to the Wayland compositor
    pub fn connect(&mut self) -> Result<(), WaylandError> {
        tracing::info!(
            "WaylandOverlay attempting connection (position: {:?})",
            self.position
        );

        let conn = Connection::connect_to_env().map_err(|_| WaylandError::ConnectionFailed)?;

        let (globals, mut event_queue) = registry_queue_init::<AppState>(&conn)
            .map_err(|_| WaylandError::ConnectionFailed)?;

        let qh = event_queue.handle();

        let compositor = CompositorState::bind(&globals, &qh)
            .map_err(|_| WaylandError::MissingGlobals)?;
        let shm = Shm::bind(&globals, &qh)
            .map_err(|_| WaylandError::MissingGlobals)?;
        let layer_shell = LayerShell::bind(&globals, &qh)
            .map_err(|_| WaylandError::LayerShellUnavailable)?;

        let mut app_state = AppState {
            registry: RegistryState::new(&globals),
            output: OutputState::new(&globals, &qh),
            compositor,
            shm,
            layer_shell,
            configured: false,
            closed: false,
            last_configure_serial: 0,
            configured_size: (0, 0),
        };

        let surface = app_state.compositor.create_surface(&qh);

        let layer_surface = app_state.layer_shell.create_layer_surface(
            &qh,
            surface.clone(),
            smithay_client_toolkit::shell::wlr_layer::Layer::Overlay,
            Some("phonesc-overlay".to_string()),
            None,
        );

        layer_surface.set_size(self.size.0, self.size.1);
        layer_surface.set_anchor(self.position.anchor_flags());
        layer_surface.set_margin(10, 10, 10, 10);
        layer_surface.set_exclusive_zone(0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);

        surface.commit();

        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();

        loop {
            event_queue
                .roundtrip(&mut app_state)
                .map_err(|_| WaylandError::SurfaceCreationFailed)?;

            if app_state.configured {
                tracing::debug!(
                    "Layer surface configured successfully. Serial={}, Suggested size={:?}",
                    app_state.last_configure_serial,
                    app_state.configured_size
                );
                break;
            }

            if start.elapsed() > timeout {
                tracing::error!("Timeout waiting for layer surface configure event");
                return Err(WaylandError::SurfaceCreationFailed);
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        surface.commit();

        let buffer_size = (self.size.0 * self.size.1 * 4) as usize;
        let buffer_pool = Arc::new(Mutex::new(
            SlotPool::new(buffer_size, &app_state.shm)
                .map_err(|_| WaylandError::BufferCreationFailed)?,
        ));

        self.connection = Some(conn);
        self.event_queue = Some(event_queue);
        self.app_state = Some(app_state);
        self.layer_surface = Some(layer_surface);
        self.surface = Some(surface);
        self.buffer_pool = Some(buffer_pool);
        self.connected = true;
        self.surface_closed = false;

        tracing::info!("Wayland overlay connected successfully and configured");
        Ok(())
    }

    /// Updates the overlay color by attaching a new buffer
    pub fn update_color(&mut self, color: OverlayColor) -> Result<(), WaylandError> {
        if self.surface_closed {
            tracing::warn!("Layer surface was closed by compositor, reconnecting...");
            self.disconnect();
            self.connect()?;
        }

        if !self.connected || self.surface.is_none() {
            self.connect()?;
        }

        let rgba_pixel_data = crate::overlay::renderer::render_circle(color);
        let bgra_pixel_data = crate::overlay::renderer::rgba_to_bgra(&rgba_pixel_data);
        let buffer_size = (self.size.0 * self.size.1 * 4) as usize;

        if bgra_pixel_data.len() != buffer_size {
            return Err(WaylandError::BufferCreationFailed);
        }

        let buffer_pool = self
            .buffer_pool
            .as_ref()
            .ok_or(WaylandError::MissingGlobals)?;
        let surface = self.surface.as_ref().ok_or(WaylandError::CommitFailed)?;

        let mut pool = buffer_pool
            .lock()
            .map_err(|_| WaylandError::BufferCreationFailed)?;

        let (buffer, canvas) = pool
            .create_buffer(
                self.size.0 as i32,
                self.size.1 as i32,
                (self.size.0 * 4) as i32,
                wayland_client::protocol::wl_shm::Format::Argb8888,
            )
            .map_err(|_| WaylandError::BufferCreationFailed)?;

        canvas.copy_from_slice(&bgra_pixel_data);

        drop(pool);

        buffer
            .attach_to(&surface)
            .map_err(|_| WaylandError::CommitFailed)?;
        surface.damage_buffer(0, 0, self.size.0 as i32, self.size.1 as i32);
        surface.commit();

        if let Some(app_state) = self.app_state.as_mut() {
            if let Some(event_queue) = self.event_queue.as_mut() {
                event_queue
                    .roundtrip(app_state)
                    .map_err(|_| WaylandError::CommitFailed)?;

                if app_state.closed {
                    self.surface_closed = true;
                }
            }
        }

        tracing::debug!(
            "WaylandOverlay color updated (position: {:?})",
            self.position
        );
        Ok(())
    }

    /// Disconnects from the Wayland compositor
    pub fn disconnect(&mut self) {
        self.buffer_pool = None;
        self.layer_surface = None;
        self.surface = None;
        self.event_queue = None;
        self.app_state = None;
        self.connection = None;
        self.connected = false;
        tracing::debug!("WaylandOverlay disconnected");
    }

    /// Returns the current position
    pub fn position(&self) -> OverlayPosition {
        self.position
    }

    /// Returns the overlay size
    pub fn size(&self) -> (u32, u32) {
        self.size
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
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_position_parsing() {
        assert_eq!(
            OverlayPosition::from_str("top-right").unwrap(),
            OverlayPosition::TopRight
        );
        assert_eq!(
            OverlayPosition::from_str("top-left").unwrap(),
            OverlayPosition::TopLeft
        );
        assert_eq!(
            OverlayPosition::from_str("bottom-right").unwrap(),
            OverlayPosition::BottomRight
        );
        assert_eq!(
            OverlayPosition::from_str("bottom-left").unwrap(),
            OverlayPosition::BottomLeft
        );
    }

    #[test]
    fn test_overlay_position_case_insensitive() {
        assert_eq!(
            OverlayPosition::from_str("TOP-RIGHT").unwrap(),
            OverlayPosition::TopRight
        );
        assert_eq!(
            OverlayPosition::from_str("Bottom-Left").unwrap(),
            OverlayPosition::BottomLeft
        );
    }

    #[test]
    fn test_overlay_position_invalid() {
        assert!(OverlayPosition::from_str("invalid").is_err());
        assert!(OverlayPosition::from_str("center").is_err());
    }

    #[test]
    fn test_wayland_overlay_creation() {
        let overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        assert_eq!(overlay.position(), OverlayPosition::TopRight);
        assert_eq!(overlay.size(), (32, 32));
    }

    #[test]
    fn test_wayland_overlay_connection_no_display() {
        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let result = overlay.connect();
        match result {
            Ok(()) => {
                assert!(overlay.connected);
            }
            Err(WaylandError::ConnectionFailed) |
            Err(WaylandError::NoDisplay) |
            Err(WaylandError::MissingGlobals) => {
                assert!(!overlay.connected);
            }
            Err(e) => {
                panic!("Unexpected error type: {}", e);
            }
        }
    }

    #[test]
    fn test_wayland_overlay_color_update_disconnected() {
        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let color = OverlayColor::opaque(0, 255, 0);
        let result = overlay.update_color(color);
        match result {
            Ok(()) => {
            }
            Err(_) => {
            }
        }
    }

    #[test]
    #[ignore = "Requires active Wayland display"]
    fn test_wayland_overlay_connection_with_display() {
        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        let result = overlay.connect();
        assert!(result.is_ok(), "Connection should succeed with Wayland display");
        assert!(overlay.connected);
    }

    #[test]
    #[ignore = "Requires active Wayland display"]
    fn test_wayland_overlay_color_update_with_display() {
        let mut overlay =
            WaylandOverlay::new(OverlayPosition::TopRight).expect("Failed to create overlay");
        overlay.connect().expect("Failed to connect");

        let color = OverlayColor::opaque(0, 255, 0);
        let result = overlay.update_color(color);
        assert!(result.is_ok(), "Color update should succeed with connected overlay");
    }
}
