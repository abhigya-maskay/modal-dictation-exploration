use thiserror::Error;

/// Error types for Wayland operations
#[derive(Debug, Error)]
pub enum WaylandError {
    #[error("Failed to connect to Wayland display")]
    ConnectionFailed,

    #[error("Failed to create layer shell surface")]
    SurfaceCreationFailed,

    #[error("Failed to create shared memory buffer")]
    BufferCreationFailed,

    #[error("Failed to commit surface")]
    CommitFailed,

    #[error("Missing Wayland globals")]
    MissingGlobals,

    #[error("Layer shell not available")]
    LayerShellUnavailable,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
