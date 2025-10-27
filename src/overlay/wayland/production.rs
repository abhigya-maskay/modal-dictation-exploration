//! Production Wayland Protocol Implementation
//!
//! This module implements the `WaylandProtocol` trait using actual Wayland primitives
//! via smithay-client-toolkit. It handles:
//! - Connection to Wayland compositor
//! - wlr-layer-shell surface creation and configuration
//! - Shared memory buffer management
//! - Event processing for configure and closed events
//!
//! # Testing Strategy
//!
//! This module uses a three-tier testing approach:
//!
//! ## 1. Smoke Tests (src/overlay/wayland/production_tests.rs)
//! These run **without a compositor** and verify:
//! - Instantiation doesn't panic
//! - Error paths work correctly (connection failures)
//! - State management is consistent
//! - Safe operation on disconnected state
//!
//! Smoke tests catch regressions in error handling and state management
//! that can be tested without actual Wayland operations.
//!
//! ## 2. Integration Tests (conditional tests in overlay.rs)
//! These run **only when WAYLAND_DISPLAY is set** and verify:
//! - Successful connection establishment
//! - Surface creation and configuration
//! - Buffer creation and attachment
//! - Event processing
//!
//! Integration tests verify the complete production code path but require
//! a real compositor.
//!
//! ## 3. Manual Verification Required
//! The following aspects cannot be automatically tested without significant
//! infrastructure or mocking complexity:
//!
//! - **Event handler callbacks** (LayerShellHandler, CompositorHandler):
//!   Manual test: Restart compositor and verify `closed` flag is set
//!
//! - **Configure event timeout** (5 second timeout in connect()):
//!   Manual test: Verify configure arrives promptly under normal conditions
//!
//! - **Layer surface visual positioning**:
//!   Manual test: Verify overlay appears at correct corner with correct size
//!
//! - **Buffer rendering correctness**:
//!   Manual test: Verify colors display correctly without artifacts
//!
//! See `TESTING.md` for detailed manual test procedures.
//!
//! # Known Limitations
//!
//! - No automated testing of event callbacks without mocking smithay
//! - No automated testing of configure timeout behavior
//! - No visual regression testing
//! - Limited compositor compatibility testing (manual verification recommended)

use super::{OverlayPosition, WaylandError};
use smithay_client_toolkit::compositor::CompositorState;
use smithay_client_toolkit::output::OutputState;
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::shell::wlr_layer::{KeyboardInteractivity, LayerShell, LayerSurface};
use smithay_client_toolkit::shm::slot::SlotPool;
use smithay_client_toolkit::shm::Shm;
use std::sync::{Arc, Mutex};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::globals::registry_queue_init;
use wayland_client::{Connection, EventQueue, QueueHandle};

/// Wayland application state using SCTK patterns
pub(super) struct AppState {
    pub registry: RegistryState,
    pub output: OutputState,
    pub compositor: CompositorState,
    pub shm: Shm,
    pub layer_shell: LayerShell,
    pub configured: bool,
    pub closed: bool,
    pub last_configure_serial: u32,
    pub configured_size: (u32, u32),
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
        _layer: &LayerSurface,
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

/// Production implementation of WaylandProtocol using actual Wayland primitives
///
/// This implementation uses smithay-client-toolkit to communicate with the
/// Wayland compositor and manage a wlr-layer-shell surface.
pub struct ProductionWaylandProtocol {
    position: OverlayPosition,
    size: (u32, u32),
    connection: Option<Connection>,
    event_queue: Option<EventQueue<AppState>>,
    app_state: Option<AppState>,
    layer_surface: Option<LayerSurface>,
    surface: Option<WlSurface>,
    buffer_pool: Option<Arc<Mutex<SlotPool>>>,
    connected: bool,
}

impl ProductionWaylandProtocol {
    /// Creates a new production Wayland protocol handler
    pub fn new(position: OverlayPosition, size: (u32, u32)) -> Self {
        Self {
            position,
            size,
            connection: None,
            event_queue: None,
            app_state: None,
            layer_surface: None,
            surface: None,
            buffer_pool: None,
            connected: false,
        }
    }
}

impl super::protocol::WaylandProtocol for ProductionWaylandProtocol {
    fn connect(&mut self, position: OverlayPosition, size: (u32, u32)) -> Result<(), WaylandError> {
        self.position = position;
        self.size = size;

        tracing::info!(
            "ProductionWaylandProtocol attempting connection (position: {:?})",
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

        tracing::info!("ProductionWaylandProtocol connected successfully");
        Ok(())
    }

    fn is_surface_closed(&self) -> bool {
        self.app_state.as_ref().map_or(false, |s| s.closed)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn update_buffer(&mut self, pixels: &[u8]) -> Result<(), WaylandError> {
        let buffer_size = (self.size.0 * self.size.1 * 4) as usize;

        if pixels.len() != buffer_size {
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

        canvas.copy_from_slice(pixels);

        drop(pool);

        buffer
            .attach_to(surface)
            .map_err(|_| WaylandError::CommitFailed)?;
        surface.damage_buffer(0, 0, self.size.0 as i32, self.size.1 as i32);
        surface.commit();

        if let Some(app_state) = self.app_state.as_mut() {
            if let Some(event_queue) = self.event_queue.as_mut() {
                event_queue
                    .roundtrip(app_state)
                    .map_err(|_| WaylandError::CommitFailed)?;
            }
        }

        tracing::debug!("ProductionWaylandProtocol buffer updated");
        Ok(())
    }

    fn disconnect(&mut self) {
        self.buffer_pool = None;
        self.layer_surface = None;
        self.surface = None;
        self.event_queue = None;
        self.app_state = None;
        self.connection = None;
        self.connected = false;
        tracing::debug!("ProductionWaylandProtocol disconnected");
    }

    fn position(&self) -> OverlayPosition {
        self.position
    }

    fn set_position(&mut self, position: OverlayPosition) {
        self.position = position;
    }
}
