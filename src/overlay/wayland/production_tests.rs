use super::production::ProductionWaylandProtocol;
use super::protocol::WaylandProtocol;
use super::{OverlayPosition, WaylandError};

/// Smoke test: ProductionWaylandProtocol instantiation
///
/// Verifies that creating a ProductionWaylandProtocol instance doesn't panic
/// and initializes with correct default state.
#[test]
fn test_production_protocol_creation() {
    let protocol = ProductionWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));
    assert_eq!(protocol.position(), OverlayPosition::TopRight);
    assert!(!protocol.is_connected(), "Should not be connected on creation");
    assert!(!protocol.is_surface_closed(), "Surface should not be marked closed on creation");
}

/// Smoke test: Connection failure when no Wayland display available
///
/// Verifies that ProductionWaylandProtocol handles connection failure gracefully
/// when no compositor is available. This tests the error path through
/// Connection::connect_to_env() and ensures proper error type is returned.
#[test]
fn test_production_protocol_connect_failure_no_display() {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        eprintln!("test_production_protocol_connect_failure_no_display: Skipping (WAYLAND_DISPLAY is set)");
        return;
    }

    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::BottomLeft, (32, 32));
    let result = protocol.connect(OverlayPosition::BottomLeft, (32, 32));

    match result {
        Err(WaylandError::ConnectionFailed) => {
            assert!(!protocol.is_connected());
        }
        Err(WaylandError::MissingGlobals) => {
            assert!(!protocol.is_connected());
        }
        Err(WaylandError::LayerShellUnavailable) => {
            assert!(!protocol.is_connected());
        }
        Ok(()) => {
            panic!("Expected connection to fail without WAYLAND_DISPLAY, but it succeeded");
        }
        Err(e) => {
            panic!("Unexpected error type: {:?}", e);
        }
    }
}

/// Smoke test: State consistency after failed connection
///
/// Verifies that after a failed connection attempt, the protocol remains in a
/// consistent state and multiple connection attempts don't cause panics or
/// leave resources in a bad state.
#[test]
fn test_production_protocol_state_consistency_after_failed_connect() {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        eprintln!("test_production_protocol_state_consistency_after_failed_connect: Skipping (WAYLAND_DISPLAY is set)");
        return;
    }

    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopLeft, (64, 64));

    let result1 = protocol.connect(OverlayPosition::TopLeft, (64, 64));
    assert!(result1.is_err(), "First connection should fail");
    assert!(!protocol.is_connected());

    let result2 = protocol.connect(OverlayPosition::TopRight, (64, 64));
    assert!(result2.is_err(), "Second connection should also fail");
    assert!(!protocol.is_connected());

    assert!(!protocol.is_surface_closed());
    assert_eq!(protocol.position(), OverlayPosition::TopRight);
}

/// Smoke test: Disconnect without connect is safe
///
/// Verifies that calling disconnect() on a never-connected protocol is safe
/// and doesn't panic or cause issues.
#[test]
fn test_production_protocol_disconnect_without_connect() {
    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::BottomRight, (32, 32));

    protocol.disconnect();

    assert!(!protocol.is_connected());
    assert!(!protocol.is_surface_closed());
}

/// Smoke test: Update buffer fails gracefully when not connected
///
/// Verifies that attempting to update the buffer without an active connection
/// returns an appropriate error rather than panicking.
#[test]
fn test_production_protocol_update_buffer_without_connect() {
    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let buffer_size = 32 * 32 * 4;
    let pixels = vec![0u8; buffer_size];

    let result = protocol.update_buffer(&pixels);

    assert!(result.is_err(), "Update buffer should fail when not connected");

    match result.unwrap_err() {
        WaylandError::MissingGlobals |
        WaylandError::CommitFailed |
        WaylandError::BufferCreationFailed => {
        }
        e => {
            panic!("Unexpected error type for disconnected update_buffer: {:?}", e);
        }
    }
}

/// Smoke test: Position management
///
/// Verifies that position getter/setter work correctly and persist through
/// state changes.
#[test]
fn test_production_protocol_position_management() {
    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));

    assert_eq!(protocol.position(), OverlayPosition::TopLeft);

    protocol.set_position(OverlayPosition::BottomRight);
    assert_eq!(protocol.position(), OverlayPosition::BottomRight);

    if std::env::var("WAYLAND_DISPLAY").is_err() {
        let _ = protocol.connect(OverlayPosition::BottomRight, (32, 32));
        assert_eq!(
            protocol.position(),
            OverlayPosition::BottomRight,
            "Position should persist after failed connect"
        );
    }
}

/// Smoke test: Invalid buffer size is rejected
///
/// Verifies that update_buffer rejects buffers with incorrect size.
#[test]
fn test_production_protocol_invalid_buffer_size() {
    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let wrong_size_pixels = vec![0u8; 100];
    let result = protocol.update_buffer(&wrong_size_pixels);

    assert!(result.is_err(), "Should reject buffer with wrong size");
}

/// Integration test: Successful connection and basic operations
///
/// This test only runs when a real Wayland compositor is available (WAYLAND_DISPLAY set).
/// It verifies the complete production code path including smithay-client-toolkit
/// integration, event handling, and buffer operations.
///
/// This is the complement to the smoke tests above - it verifies what the
/// smoke tests cannot: actual Wayland protocol interaction.
#[test]
fn test_production_protocol_full_workflow_with_compositor() {
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        eprintln!("test_production_protocol_full_workflow_with_compositor: Skipping (WAYLAND_DISPLAY not set)");
        return;
    }

    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let connect_result = protocol.connect(OverlayPosition::TopRight, (32, 32));
    assert!(
        connect_result.is_ok(),
        "Connection should succeed with compositor: {:?}",
        connect_result
    );
    assert!(protocol.is_connected());
    assert!(!protocol.is_surface_closed());

    let buffer_size = 32 * 32 * 4;
    let pixels = vec![255u8; buffer_size];
    let update_result = protocol.update_buffer(&pixels);
    assert!(
        update_result.is_ok(),
        "Buffer update should succeed: {:?}",
        update_result
    );

    protocol.disconnect();
    assert!(!protocol.is_connected());
}

/// Integration test: Position change during reconnection
///
/// Verifies that changing position parameter during connect() is properly
/// reflected in the layer surface configuration.
#[test]
fn test_production_protocol_position_change_on_connect() {
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        eprintln!("test_production_protocol_position_change_on_connect: Skipping (WAYLAND_DISPLAY not set)");
        return;
    }

    let mut protocol = ProductionWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));

    let result = protocol.connect(OverlayPosition::BottomRight, (32, 32));
    assert!(result.is_ok());
    assert_eq!(protocol.position(), OverlayPosition::BottomRight);

    protocol.disconnect();
}
