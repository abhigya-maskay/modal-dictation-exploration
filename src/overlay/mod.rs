mod backend;
mod defaults;
mod manager;
mod renderer;
mod state;
mod wayland;

pub use backend::{OverlayBackend, MockOverlayBackend, FailingMockBackend};
pub use defaults::{
    DEFAULT_AWAKE_COLOR, DEFAULT_ASLEEP_COLOR, DEFAULT_ERROR_COLOR,
    DEFAULT_AWAKE_COLOR_NAME, DEFAULT_ASLEEP_COLOR_NAME, DEFAULT_ERROR_COLOR_NAME,
};
pub use manager::{OverlayManager, parse_position_with_fallback};
pub use renderer::OverlayColor;
pub use state::{OverlayRenderState, ReconnectionState};
pub use wayland::{OverlayPosition, WaylandOverlay};

#[cfg(test)]
mod tests;
