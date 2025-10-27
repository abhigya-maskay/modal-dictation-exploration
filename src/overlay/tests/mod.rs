//! Integration tests for overlay manager
//!
//! Tests cover:
//! - State initialization and color selection
//! - Live config reload and position changes
//! - Activation state transitions
//! - Error handling and reconnection with backoff
//! - Health checks and graceful shutdown

mod helpers;

mod manager_tests;
