//! Integration tests for OverlayManager
//!
//! Covers end-to-end scenarios including config changes, activation
//! state transitions, error recovery, and graceful shutdown.

use super::helpers::{
    create_test_config_dir,
    create_default_test_config,
    create_test_overlay_manager,
    TrackedMockBackend,
};
use crate::activation::{ActivationManager, SystemState};
use crate::config::ConfigManager;
use crate::overlay::{
    OverlayBackend, OverlayColor, OverlayManager, OverlayPosition,
    MockOverlayBackend, FailingMockBackend,
};
use std::sync::Arc;

#[tokio::test]
async fn test_overlay_manager_creation_with_defaults() {
    let (_temp_dir, config_path) = create_default_test_config();
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);
    assert!(!state.has_error);
}

#[tokio::test]
async fn test_overlay_color_selection_asleep_state() {
    let (_temp_dir, config_path) = create_default_test_config();
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);
    let color = state.current_color();
    assert_eq!(color, OverlayColor::opaque(128, 128, 128));

    drop(overlay);
}

#[tokio::test]
async fn test_overlay_custom_colors() {
    let config_content = r#"
[overlay]
asleep_color = "blue"
awake_color = "yellow"
error_color = "red"
position = "bottom-left"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);
    let color = state.current_color();
    assert_eq!(color, OverlayColor::opaque(0, 0, 255));
    assert_eq!(state.config.position, "bottom-left");

    drop(overlay);
}

#[tokio::test]
async fn test_overlay_error_state_tracking() {
    let (_temp_dir, config_path) = create_default_test_config();
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let initial_state = overlay.current_state().await;
    assert!(!initial_state.has_error);

    let has_error = overlay.has_error().await;
    assert!(!has_error);

    drop(overlay);
}

#[tokio::test]
async fn test_overlay_state_initialization_preserves_config() {
    let config_content = r#"
[overlay]
asleep_color = "purple"
awake_color = "cyan"
error_color = "red"
position = "bottom-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.config.position, "bottom-right");
    assert_eq!(state.config.asleep_color, "purple");
    assert_eq!(state.config.awake_color, "cyan");

    assert_eq!(state.asleep_color, OverlayColor::opaque(128, 0, 128));
    assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 255));

    drop(overlay);
}

#[tokio::test]
async fn test_overlay_multiple_instances_independent_state() {
    let config_content1 = r#"
[overlay]
asleep_color = "blue"
awake_color = "yellow"
error_color = "red"
position = "top-left"
"#;
    let config_content2 = r#"
[overlay]
asleep_color = "green"
awake_color = "red"
error_color = "yellow"
position = "bottom-right"
"#;

    let (_temp_dir1, config_path1) = create_test_config_dir(config_content1);
    let (_temp_dir2, config_path2) = create_test_config_dir(config_content2);

    let config_mgr1 = ConfigManager::new_with_path(config_path1)
        .expect("Failed to create config manager 1");
    let config_mgr2 = ConfigManager::new_with_path(config_path2)
        .expect("Failed to create config manager 2");

    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay1 = create_test_overlay_manager(&config_mgr1, &activation_mgr);
    let overlay2 = create_test_overlay_manager(&config_mgr2, &activation_mgr);

    let state1 = overlay1.current_state().await;
    let state2 = overlay2.current_state().await;

    assert_eq!(state1.config.position, "top-left");
    assert_eq!(state2.config.position, "bottom-right");

    assert_eq!(state1.asleep_color, OverlayColor::opaque(0, 0, 255));
    assert_eq!(state2.asleep_color, OverlayColor::opaque(0, 255, 0));

    drop(overlay1);
    drop(overlay2);
}

#[tokio::test]
async fn test_overlay_color_based_on_error_state() {
    let (_temp_dir, config_path) = create_default_test_config();
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert!(!state.has_error);

    let color = state.current_color();
    assert_eq!(color, OverlayColor::opaque(128, 128, 128));

    drop(overlay);
}

#[tokio::test]
async fn test_overlay_state_with_hex_colors() {
    let config_content = "[overlay]\nasleep_color = \"#FF00FF\"\nawake_color = \"#00FF00\"\nerror_color = \"#0000FF\"\nposition = \"top-right\"\n";
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.asleep_color, OverlayColor::opaque(255, 0, 255));
    assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 0));
    assert_eq!(state.error_color, OverlayColor::opaque(0, 0, 255));

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_activation_state_change_updates_color() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);
    assert_eq!(state.current_color(), OverlayColor::opaque(128, 128, 128));

    activation_mgr.wake_via_wake_word().await;

    tokio::time::advance(std::time::Duration::from_millis(100)).await;

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Awake);
    assert_eq!(state.current_color(), OverlayColor::opaque(0, 255, 0));

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_config_change_updates_colors() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path.clone())
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);
    let mut config_rx = config_mgr.subscribe();

    activation_mgr.wake_via_wake_word().await;
    tokio::time::advance(std::time::Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    let state = overlay.current_state().await;
    assert_eq!(state.awake_color, OverlayColor::opaque(0, 255, 0));

    let new_config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "blue"
error_color = "red"
position = "top-right"
"#;
    let config_file_path = config_path.join("config.toml");
    std::fs::write(&config_file_path, new_config_content)
        .expect("Failed to write updated config");

    let change_result = tokio::time::timeout(std::time::Duration::from_secs(2), config_rx.changed()).await;
    assert!(change_result.is_ok(), "Timeout waiting for config change");
    assert!(change_result.unwrap().is_ok(), "Config change notification failed");

    let state = overlay.current_state().await;
    assert_eq!(state.awake_color, OverlayColor::opaque(0, 0, 255));

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_config_change_updates_position_triggers_reconnection() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path.clone())
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let factory_positions = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let factory_positions_clone = factory_positions.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let mut positions = factory_positions_clone.lock().unwrap();
        positions.push(position);
        drop(positions);

        MockOverlayBackend::new(position)
            .map(|backend| Box::new(backend) as Box<dyn OverlayBackend>)
    });

    let mut config_rx = config_mgr.subscribe();

    tokio::time::advance(std::time::Duration::from_millis(200)).await;
    tokio::task::yield_now().await;

    let initial_positions = factory_positions.lock().unwrap();
    assert!(!initial_positions.is_empty());
    assert_eq!(initial_positions[0], OverlayPosition::TopRight);
    drop(initial_positions);

    let new_config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "bottom-left"
"#;
    let config_file_path = config_path.join("config.toml");
    std::fs::write(&config_file_path, new_config_content)
        .expect("Failed to write updated config");

    let change_result = tokio::time::timeout(std::time::Duration::from_secs(2), config_rx.changed()).await;
    assert!(change_result.is_ok());
    assert!(change_result.unwrap().is_ok());

    let final_positions = factory_positions.lock().unwrap();
    assert!(final_positions.len() >= 2);
    assert_eq!(final_positions[0], OverlayPosition::TopRight);
    assert_eq!(final_positions[final_positions.len() - 1], OverlayPosition::BottomLeft);

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_error_state_switches_to_error_color() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let count = attempt_count_clone.clone();
        FailingMockBackend::new(position).map(move |backend| {
            let mut attempts = count.lock().unwrap();
            *attempts += 1;
            let attempt_num = *attempts;
            drop(attempts);

            if attempt_num == 1 {
                Box::new(backend.fail_update_color_n_times(1000)) as Box<dyn OverlayBackend>
            } else {
                Box::new(backend) as Box<dyn OverlayBackend>
            }
        })
    });

    tokio::time::advance(std::time::Duration::from_millis(150)).await;

    let state = overlay.current_state().await;
    assert!(state.has_error);
    assert_eq!(state.current_color(), OverlayColor::opaque(255, 0, 0));

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_reconnection_after_initial_failure() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let factory_call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let factory_call_count_clone = factory_call_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let mut count = factory_call_count_clone.lock().unwrap();
        *count += 1;

        FailingMockBackend::new(position)
            .map(|backend| {
                if *count <= 2 {
                    Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
                } else {
                    Box::new(backend) as Box<dyn OverlayBackend>
                }
            })
    });

    tokio::time::advance(std::time::Duration::from_millis(500)).await;

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_reconnection_exponential_backoff() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let count = attempt_count_clone.clone();
        FailingMockBackend::new(position)
            .map(move |backend| {
                let mut attempts = count.lock().unwrap();
                *attempts += 1;
                drop(attempts);

                Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
            })
    });

    for _ in 0..3 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    for _ in 0..7 {
        tokio::time::advance(std::time::Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
    }

    let times = attempt_count.lock().unwrap();
    assert!(*times >= 2);

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_successful_reconnect_resets_backoff() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let should_fail = std::sync::Arc::new(std::sync::Mutex::new(true));
    let should_fail_clone = should_fail.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        FailingMockBackend::new(position)
            .map(|backend| {
                let fail = should_fail_clone.lock().unwrap();
                if *fail {
                    Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
                } else {
                    Box::new(backend) as Box<dyn OverlayBackend>
                }
            })
    });

    for _ in 0..3 {
        tokio::time::advance(std::time::Duration::from_millis(50)).await;
        tokio::task::yield_now().await;
    }

    *should_fail.lock().unwrap() = false;

    for _ in 0..2 {
        tokio::time::advance(std::time::Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
    }

    let state = overlay.current_state().await;
    assert!(!state.has_error);

    let status = overlay.reconnection_status().await;
    assert_eq!(status.attempt_count, 0);
    assert_eq!(status.next_backoff_duration.as_millis(), 1000);

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_health_check_detects_broken_overlay() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let health_fail_count = std::sync::Arc::new(std::sync::Mutex::new(false));
    let health_fail_count_clone = health_fail_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let should_fail = *health_fail_count_clone.lock().unwrap();
        FailingMockBackend::new(position)
            .map(|backend| {
                if should_fail {
                    Box::new(backend.fail_update_color_n_times(1)) as Box<dyn OverlayBackend>
                } else {
                    Box::new(backend) as Box<dyn OverlayBackend>
                }
            })
    });

    let state = overlay.current_state().await;
    assert!(!state.has_error);

    *health_fail_count.lock().unwrap() = true;

    tokio::time::advance(std::time::Duration::from_millis(2500)).await;

    let state = overlay.current_state().await;
    assert!(state.has_error);

    drop(overlay);
}

#[tokio::test]
async fn test_reconnection_status_initially_no_failures() {
    let (_temp_dir, config_path) = create_default_test_config();
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let status = overlay.reconnection_status().await;
    assert_eq!(status.attempt_count, 0);
    assert_eq!(status.next_backoff_duration.as_millis(), 1000);
    assert!(!status.ready_to_retry);

    drop(overlay);
}

#[tokio::test(start_paused = true)]
async fn test_reconnection_status_tracks_failures() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let attempt_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let attempt_count_clone = attempt_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let count = attempt_count_clone.clone();
        FailingMockBackend::new(position)
            .map(move |backend| {
                let mut attempts = count.lock().unwrap();
                *attempts += 1;
                drop(attempts);

                Box::new(backend.fail_connect_n_times(1)) as Box<dyn OverlayBackend>
            })
    });

    for _ in 0..3 {
        tokio::time::advance(std::time::Duration::from_millis(50)).await;
        tokio::task::yield_now().await;
    }

    let status = overlay.reconnection_status().await;
    assert!(status.attempt_count > 0);
    assert_eq!(status.next_backoff_duration.as_millis(), 1000);

    drop(overlay);
}

/// Integration test: verifies config file changes propagate through ConfigManager
/// to OverlayManager and result in actual backend.update_color() calls
#[tokio::test(start_paused = true)]
async fn test_config_file_change_propagates_to_backend() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);

    let color_history = Arc::new(std::sync::Mutex::new(Vec::new()));
    let color_history_clone = color_history.clone();

    let config_mgr = ConfigManager::new_with_path(config_path.clone())
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let _overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        TrackedMockBackend::new(position, color_history_clone.clone())
            .map(|backend| Box::new(backend) as Box<dyn OverlayBackend>)
    });

    let mut config_rx = config_mgr.subscribe();

    for _ in 0..5 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let colors = color_history.lock().unwrap().clone();
    assert!(!colors.is_empty());
    assert_eq!(colors[0], OverlayColor::opaque(128, 128, 128));

    color_history.lock().unwrap().clear();

    let new_config_content = r#"
[overlay]
asleep_color = "blue"
awake_color = "yellow"
error_color = "red"
position = "bottom-left"
"#;
    let config_file_path = config_path.join("config.toml");
    std::fs::write(&config_file_path, new_config_content)
        .expect("Failed to write updated config");

    let change_result = tokio::time::timeout(std::time::Duration::from_secs(2), config_rx.changed()).await;
    assert!(change_result.is_ok());
    assert!(change_result.unwrap().is_ok());

    let final_colors = color_history.lock().unwrap().clone();
    assert!(!final_colors.is_empty());

    let last_color = final_colors.last().unwrap();
    assert_eq!(*last_color, OverlayColor::opaque(0, 0, 255));

    assert!(final_colors.len() >= 1);
}

/// Test that dropping managers doesn't cause the overlay task to spin in a hot loop
///
/// This test verifies that when ConfigManager and ActivationManager are dropped,
/// the overlay task gracefully exits instead of spinning on closed watch channels.
#[tokio::test(start_paused = true)]
async fn test_graceful_shutdown_on_manager_drop() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let factory_call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
    let factory_call_count_clone = factory_call_count.clone();

    let overlay = OverlayManager::new_with_factory(&config_mgr, &activation_mgr, move |position| {
        let mut count = factory_call_count_clone.lock().unwrap();
        *count += 1;
        drop(count);

        MockOverlayBackend::new(position)
            .map(|backend| Box::new(backend) as Box<dyn OverlayBackend>)
    });

    for _ in 0..5 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let initial_calls = *factory_call_count.lock().unwrap();
    assert!(initial_calls > 0);

    drop(config_mgr);
    drop(activation_mgr);

    for _ in 0..5 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let calls_after_drop = *factory_call_count.lock().unwrap();

    assert!(
        calls_after_drop - initial_calls <= 2,
        "Factory should not be called repeatedly after managers dropped (initial: {}, after: {})",
        initial_calls,
        calls_after_drop
    );

    drop(overlay);
}

/// Test that the overlay task exits when ConfigManager is dropped
///
/// Verifies that channel closure from ConfigManager drop is detected and handled.
#[tokio::test(start_paused = true)]
async fn test_task_exits_when_config_manager_dropped() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    for _ in 0..5 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);

    drop(config_mgr);

    for _ in 0..10 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);

    drop(overlay);
    drop(activation_mgr);
}

/// Test that the overlay task exits when ActivationManager is dropped
///
/// Verifies that channel closure from ActivationManager drop is detected and handled.
#[tokio::test(start_paused = true)]
async fn test_task_exits_when_activation_manager_dropped() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "top-right"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    for _ in 0..5 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);

    drop(activation_mgr);

    for _ in 0..10 {
        tokio::time::advance(std::time::Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
    }

    let state = overlay.current_state().await;
    assert_eq!(state.system_state, SystemState::Asleep);

    drop(overlay);
    drop(config_mgr);
}

#[tokio::test]
async fn test_invalid_position_uses_fallback_and_caches() {
    let config_content = r#"
[overlay]
asleep_color = "gray"
awake_color = "green"
error_color = "red"
position = "invalid-position"
"#;
    let (_temp_dir, config_path) = create_test_config_dir(config_content);
    let config_mgr = ConfigManager::new_with_path(config_path)
        .expect("Failed to create config manager");
    let activation_mgr = Arc::new(ActivationManager::new(300));

    let overlay = create_test_overlay_manager(&config_mgr, &activation_mgr);

    let state = overlay.current_state().await;
    assert_eq!(state.config.position, "invalid-position");

    assert_eq!(state.cached_position, OverlayPosition::TopRight);

    drop(overlay);
}
