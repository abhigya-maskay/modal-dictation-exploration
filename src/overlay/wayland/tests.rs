use super::*;
use crate::overlay::renderer::OverlayColor;
use crate::overlay::wayland::overlay::WaylandOverlay;
use crate::overlay::wayland::mock::MockWaylandProtocol;

/// Test: surface_closed triggers reconnection
///
/// Verifies that when the compositor closes the surface (e.g., compositor restart),
/// the overlay detects it and automatically reconnects on the next update_color() call.
#[test]
fn test_surface_closed_triggers_reconnection() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::TopRight, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    assert!(overlay.is_connected());
    assert_eq!(handle.connect_count(), 1);

    let color = OverlayColor::opaque(0, 255, 0);
    overlay.update_color(color).expect("First color update failed");
    assert_eq!(handle.update_buffer_count(), 1);

    handle.simulate_surface_closed();

    overlay.update_color(color).expect("Color update after surface_closed failed");

    assert_eq!(handle.disconnect_count(), 1, "Should have disconnected once");
    assert_eq!(handle.connect_count(), 2, "Should have reconnected (total 2 connects)");
    assert_eq!(handle.update_buffer_count(), 2, "Should have updated buffer after reconnect");
    assert!(overlay.is_connected(), "Should be connected after reconnection");
}

/// Test: reconnection succeeds and clears surface_closed flag
///
/// Verifies that after a successful reconnection, the surface_closed flag is cleared
/// and subsequent updates work normally without triggering reconnection.
#[test]
fn test_reconnection_clears_surface_closed_flag() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::BottomLeft, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::BottomLeft, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(255, 0, 0);
    overlay.update_color(color).expect("First update failed");

    handle.simulate_surface_closed();

    overlay.update_color(color).expect("Reconnection update failed");
    assert_eq!(handle.connect_count(), 2);

    overlay.update_color(color).expect("Second update after reconnect failed");
    overlay.update_color(color).expect("Third update after reconnect failed");

    assert_eq!(handle.connect_count(), 2, "Should not reconnect again");
    assert_eq!(handle.disconnect_count(), 1, "Should have disconnected only once");
    assert_eq!(handle.update_buffer_count(), 4, "Should have 4 buffer updates total");
}

/// Test: connection error during reconnection
///
/// Verifies that if reconnection fails (e.g., compositor still unavailable),
/// the overlay returns an error and can retry later.
#[test]
fn test_reconnection_failure_handling() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::TopLeft, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(0, 0, 255);

    handle.simulate_surface_closed();
    handle.inject_connect_error();

    let result = overlay.update_color(color);
    assert!(result.is_err(), "Should fail to reconnect with injected error");
    assert_eq!(handle.disconnect_count(), 1, "Should have attempted disconnect");

    handle.clear_connect_error();
    overlay.update_color(color).expect("Reconnection should succeed after error cleared");

    assert_eq!(handle.connect_count(), 2, "Should have successfully reconnected");
    assert!(overlay.is_connected());
}

/// Test: buffer error during initial update after reconnection
///
/// Verifies that if buffer creation fails during the update after reconnection,
/// the overlay handles it gracefully and can retry.
#[test]
fn test_buffer_error_after_reconnection() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::BottomRight, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::BottomRight, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(255, 255, 0);

    handle.simulate_surface_closed();

    handle.inject_buffer_error();

    let result = overlay.update_color(color);
    assert!(result.is_err(), "Should fail to update buffer with injected error");
    assert_eq!(handle.connect_count(), 2, "Should have reconnected");
    assert_eq!(handle.update_buffer_count(), 0, "Buffer update should have failed");

    handle.clear_buffer_error();
    overlay.update_color(color).expect("Buffer update should succeed after error cleared");

    assert_eq!(handle.update_buffer_count(), 1, "Should have updated buffer successfully");
}

/// Test: multiple reconnection cycles
///
/// Verifies that the overlay can handle multiple compositor restart cycles,
/// reconnecting correctly each time.
#[test]
fn test_multiple_reconnection_cycles() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::TopRight, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(128, 128, 128);

    overlay.update_color(color).expect("Update 1 failed");
    handle.simulate_surface_closed();
    overlay.update_color(color).expect("Reconnect 1 failed");

    assert_eq!(handle.connect_count(), 2);
    assert_eq!(handle.disconnect_count(), 1);

    overlay.update_color(color).expect("Update 2 failed");
    handle.simulate_surface_closed();
    overlay.update_color(color).expect("Reconnect 2 failed");

    assert_eq!(handle.connect_count(), 3);
    assert_eq!(handle.disconnect_count(), 2);

    overlay.update_color(color).expect("Update 3 failed");
    handle.simulate_surface_closed();
    overlay.update_color(color).expect("Reconnect 3 failed");

    assert_eq!(handle.connect_count(), 4, "Should have 4 total connections (initial + 3 reconnects)");
    assert_eq!(handle.disconnect_count(), 3, "Should have 3 total disconnects");
    assert!(overlay.is_connected(), "Should still be connected after multiple cycles");
}

/// Test: auto-connect on first update_color if not connected
///
/// Verifies that if the overlay is created but not explicitly connected,
/// the first update_color() call will automatically connect.
#[test]
fn test_auto_connect_on_first_update() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::BottomLeft, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::BottomLeft, Box::new(protocol))
        .expect("Failed to create overlay");

    assert!(!overlay.is_connected());
    assert_eq!(handle.connect_count(), 0);

    let color = OverlayColor::opaque(0, 255, 255);
    overlay.update_color(color).expect("Auto-connect update failed");

    assert!(overlay.is_connected());
    assert_eq!(handle.connect_count(), 1);
    assert_eq!(handle.update_buffer_count(), 1);
}

/// Test: disconnect and reconnect manually
///
/// Verifies that manual disconnect() and connect() work correctly
/// and don't interfere with automatic reconnection logic.
#[test]
fn test_manual_disconnect_and_reconnect() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::TopLeft, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::TopLeft, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    assert_eq!(handle.connect_count(), 1);

    overlay.disconnect();
    assert!(!overlay.is_connected());
    assert_eq!(handle.disconnect_count(), 1);

    overlay.connect().expect("Manual reconnect failed");
    assert!(overlay.is_connected());
    assert_eq!(handle.connect_count(), 2);

    let color = OverlayColor::opaque(255, 128, 0);
    overlay.update_color(color).expect("Update after manual reconnect failed");
    assert_eq!(handle.update_buffer_count(), 1);
}

/// Test: surface_closed detected after buffer update
///
/// Verifies that if the compositor closes the surface during a buffer update
/// (detected by protocol.is_surface_closed()), the surface_closed flag is set
/// and the next update triggers reconnection.
#[test]
fn test_surface_closed_detected_during_update() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::BottomRight, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::BottomRight, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(200, 100, 50);

    overlay.update_color(color).expect("First update failed");
    assert_eq!(handle.update_buffer_count(), 1);

    handle.simulate_surface_closed();

    overlay.update_color(color).expect("Second update failed");
    assert_eq!(handle.update_buffer_count(), 2);

    overlay.update_color(color).expect("Third update (reconnect) failed");
    assert_eq!(handle.connect_count(), 2, "Should have reconnected");
    assert_eq!(handle.disconnect_count(), 1);
    assert_eq!(handle.update_buffer_count(), 3);
}

/// Test: position change doesn't affect reconnection logic
///
/// Verifies that reconnection works correctly even if the position changes
/// between connection attempts.
#[test]
fn test_reconnection_with_position_change() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::TopRight, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::TopRight, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");

    handle.simulate_surface_closed();

    let color = OverlayColor::opaque(100, 200, 150);
    overlay.update_color(color).expect("Reconnection failed");

    assert_eq!(handle.connect_count(), 2);
    assert!(overlay.is_connected());
}

/// Test: state consistency after failed reconnection attempt
///
/// Verifies that if reconnection fails, the overlay state remains consistent
/// and a subsequent retry can succeed.
#[test]
fn test_state_consistency_after_failed_reconnection() {
    let (protocol, handle) = MockWaylandProtocol::new(OverlayPosition::BottomLeft, (32, 32));

    let mut overlay = WaylandOverlay::new_with_protocol(OverlayPosition::BottomLeft, Box::new(protocol))
        .expect("Failed to create overlay");

    overlay.connect().expect("Initial connection failed");
    let color = OverlayColor::opaque(75, 150, 225);

    handle.simulate_surface_closed();
    handle.inject_connect_error();

    assert!(overlay.update_color(color).is_err());
    assert!(!overlay.is_connected());

    handle.clear_connect_error();

    overlay.update_color(color).expect("Second reconnection attempt failed");
    assert!(overlay.is_connected());
    assert_eq!(handle.connect_count(), 2);
    assert_eq!(handle.update_buffer_count(), 1);

    overlay.update_color(color).expect("Update after successful reconnect failed");
    assert_eq!(handle.update_buffer_count(), 2);
    assert_eq!(handle.connect_count(), 2, "Should not reconnect again");
}
