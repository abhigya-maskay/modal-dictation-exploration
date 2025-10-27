# Wayland Overlay Testing Strategy

This document explains the testing approach for the Wayland overlay implementation, what is covered by automated tests, and what requires manual verification.

## Testing Architecture

The Wayland overlay uses a **trait-based abstraction** (`WaylandProtocol`) that enables two complementary testing strategies:

1. **Mock-based testing**: Using `MockWaylandProtocol` to test reconnection logic, error handling, and state management without requiring a compositor
2. **Production smoke testing**: Testing `ProductionWaylandProtocol` error paths and initialization without a compositor
3. **Integration testing**: Testing actual Wayland protocol interaction when a compositor is available

## Test Organization

### 1. Mock-Based Reconnection Tests (`tests.rs`)
**Purpose**: Comprehensive testing of reconnection logic and state management
**Requires**: No Wayland compositor (runs in CI)
**Coverage**: 11 tests covering all reconnection scenarios

These tests verify:
- Surface closed detection and automatic reconnection
- Multiple reconnection cycles
- Connection error handling during reconnection
- Buffer error handling during reconnection
- State consistency after failed operations
- Auto-connect on first update
- Manual disconnect/reconnect cycles

**Why this works**: The `MockWaylandProtocol` simulates compositor behavior, allowing us to trigger surface_closed events, inject errors, and verify the overlay's response without needing real Wayland infrastructure.

### 2. Production Protocol Smoke Tests (`production_tests.rs`)
**Purpose**: Verify `ProductionWaylandProtocol` behavior without a compositor
**Requires**: No Wayland compositor (runs in CI)
**Coverage**: 9 tests covering instantiation and error paths

These tests verify:
- ✅ Protocol instantiation doesn't panic
- ✅ Connection failure is handled gracefully (no WAYLAND_DISPLAY)
- ✅ State consistency after failed connection attempts
- ✅ Disconnect safety without prior connection
- ✅ Buffer update fails appropriately when disconnected
- ✅ Position management API works correctly
- ✅ Invalid buffer size is rejected
- ✅ Integration test with compositor (when available)
- ✅ Position changes during reconnection

**Why this matters**: These tests catch regressions in the production code's error handling and state management, even though they can't verify successful Wayland operations.

### 3. Production Integration Tests (`overlay.rs::tests`)
**Purpose**: Verify actual Wayland protocol interaction
**Requires**: Real Wayland compositor (WAYLAND_DISPLAY set)
**Coverage**: 3 tests + 2 smoke tests

These tests verify:
- WaylandOverlay instantiation
- Connection error handling without compositor
- Auto-connect error handling without compositor
- **[COMPOSITOR REQUIRED]** Successful connection establishment
- **[COMPOSITOR REQUIRED]** Complete buffer update workflow

**Why this matters**: These tests verify the complete production path including smithay-client-toolkit integration, but they only run when WAYLAND_DISPLAY is set.

## What Is Automatically Tested

### Without a Compositor (CI-friendly)
- ✅ All reconnection logic and state transitions (via mock)
- ✅ Error handling when connection fails
- ✅ State consistency after errors
- ✅ Safe operation on disconnected state
- ✅ Position management
- ✅ Buffer validation

### With a Compositor (manual or automated with headless compositor)
- ✅ Successful Wayland connection establishment
- ✅ wlr-layer-shell surface creation and configuration
- ✅ Buffer creation via smithay SlotPool
- ✅ Buffer attachment and surface commit
- ✅ Basic event processing

## What Requires Manual Verification

The following aspects of `ProductionWaylandProtocol` **cannot** be automatically tested without either:
- Mocking smithay-client-toolkit internals (high maintenance burden)
- Running a headless compositor in CI (complexity overhead)

Manual verification is needed for:

### 1. Event Handler Callbacks
**Location**: `production.rs:56-86` (LayerShellHandler)
**What to verify**:
- `closed()` callback correctly sets `closed` flag when compositor restarts
- `configure()` callback correctly processes surface configuration
- Serial and size are properly stored from configure events

**Test procedure**:
```bash
# Start the application with overlay enabled
cargo run

# Restart your compositor (e.g., sway reload)
swaymsg reload

# Verify: Check logs for "Layer surface closed by compositor"
# Verify: Overlay should reconnect automatically
```

### 2. Configure Event Timeout
**Location**: `production.rs:271-295`
**What to verify**:
- Configure event arrives within 5 second timeout
- Timeout error is returned if configure never arrives
- Loop properly exits on configure

**Test procedure**:
```bash
# Normal case - should complete quickly
cargo test test_production_protocol_full_workflow_with_compositor -- --nocapture

# Check logs show: "Layer surface configured successfully"
```

### 3. Layer Surface Configuration
**Location**: `production.rs:263-268`
**What to verify**:
- Surface appears at correct screen corner
- Size is correct (32x32)
- Margins are applied (10px all sides)
- No keyboard interaction
- Overlay layer is used

**Test procedure**:
```bash
# Run application
cargo run

# Visually verify:
# - Indicator appears in correct corner
# - Size looks correct
# - Doesn't capture keyboard input
# - Appears above normal windows
```

### 4. Buffer Operations
**Location**: `production.rs:326-373`
**What to verify**:
- Buffer is created with correct size and format (ARGB8888)
- Pixels are copied correctly to shared memory
- Buffer attachment succeeds
- Surface damage is applied correctly
- Commit succeeds

**Test procedure**:
```bash
# Run with color changes
cargo run

# Trigger activation state changes
# Verify: Colors change visually in the overlay
# Verify: No rendering artifacts or corruption
```

### 5. Compositor-Specific Compatibility
**What to verify**:
- Works with different compositors (Sway, Hyprland, etc.)
- Handles compositor restart gracefully
- No crashes on compositor disconnect

**Test procedure**:
```bash
# Test with Sway
sway
cargo run
# Verify: Overlay appears and works

# Test compositor restart
swaymsg reload
# Verify: Overlay reconnects (check logs)

# Test with Hyprland (if available)
Hyprland
cargo run
# Verify: Overlay appears and works
```

## Running Tests

### Run all automated tests (no compositor required)
```bash
cargo test overlay::wayland
```

### Run only smoke tests (guaranteed to work without compositor)
```bash
unset WAYLAND_DISPLAY
cargo test overlay::wayland::production_tests
cargo test overlay::wayland::overlay::tests::test_production_connect_fails_without_compositor
```

### Run integration tests (compositor required)
```bash
# Ensure WAYLAND_DISPLAY is set
cargo test overlay::wayland::production_tests::test_production_protocol_full_workflow_with_compositor
cargo test overlay::wayland::overlay::tests::test_production_connection_succeeds_with_compositor
cargo test overlay::wayland::overlay::tests::test_production_color_update_succeeds_with_compositor
```

### Run all tests including mock-based tests
```bash
cargo test overlay::wayland
```

## Adding New Tests

### When to add a smoke test
Add a smoke test in `production_tests.rs` when:
- Testing error handling or validation logic
- Testing state management
- Testing behavior that doesn't require actual Wayland operations
- You want CI to catch regressions automatically

### When to add an integration test
Add an integration test in `overlay.rs::tests` when:
- Testing successful Wayland protocol operations
- Testing compositor interaction
- Testing buffer rendering
- The test requires WAYLAND_DISPLAY to be meaningful

### When to use mock-based testing
Add a mock test in `tests.rs` when:
- Testing reconnection logic
- Testing complex state transitions
- Testing error recovery scenarios
- You need fine-grained control over protocol behavior

## Test Coverage Summary

| Component | Smoke Tests | Integration Tests | Mock Tests | Manual Verification |
|-----------|-------------|-------------------|------------|---------------------|
| Connection establishment | Error path | Success path | N/A | Compositor compatibility |
| Connection failure handling | ✅ | ✅ | ✅ | N/A |
| Surface creation | N/A | ✅ | N/A | Visual verification |
| Configure event handling | N/A | ✅ | N/A | Timeout scenarios |
| Layer surface config | N/A | N/A | N/A | ✅ Required |
| Buffer creation | Error path | Success path | N/A | Format/rendering |
| Buffer update | Error path | Success path | ✅ | Visual verification |
| Event processing | N/A | N/A | N/A | ✅ Required |
| Reconnection logic | N/A | N/A | ✅ | Compositor restart |
| State management | ✅ | ✅ | ✅ | N/A |
| Position management | ✅ | ✅ | ✅ | Visual verification |

## Known Limitations

1. **No automated testing of event callbacks**: The `LayerShellHandler` and `CompositorHandler` implementations are not automatically tested. Manual verification required.

2. **No automated testing of configure timeout**: The 5-second timeout in `connect()` is not automatically tested. Manual verification recommended.

3. **No visual regression testing**: Color rendering and visual appearance must be verified manually.

4. **Limited compositor coverage**: Automated tests only run against the compositor in the development environment. Testing with multiple compositors requires manual effort.

## Future Improvements

Potential enhancements to improve test coverage:

1. **Headless compositor in CI**: Set up a headless wlroots-based compositor (like `cage` or `tinywl`) in CI to enable automated integration testing.

2. **Mock smithay-client-toolkit**: Create mocks for smithay types to test event handlers without a compositor (high maintenance burden).

3. **Visual regression testing**: Add screenshot-based testing to verify rendering correctness.

4. **Compositor matrix testing**: Test against multiple compositors automatically (Sway, Hyprland, etc.).

5. **Property-based testing**: Use proptest to generate random event sequences and verify state consistency.
