mod activation;
mod config;
mod overlay;

use std::sync::Arc;
use activation::ActivationManager;
use config::ConfigManager;
use overlay::OverlayManager;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let manager = match ConfigManager::new() {
        Ok(mgr) => mgr,
        Err(e) => {
            tracing::error!("Failed to initialize ConfigManager: {}", e);
            return;
        }
    };

    let config = manager.current();

    tracing::info!("Configuration loaded successfully");
    tracing::debug!("Config: {:?}", config);

    let activation = Arc::new(ActivationManager::new(config.auto_sleep_timeout_secs));
    tracing::info!("ActivationManager initialized");

    const DEMO_GRACE_PERIOD_SECS: u64 = 3;
    let activation_demo = activation.clone();
    let mut config_rx_demo = manager.subscribe();

    tokio::spawn(async move {
        loop {
            let current_config = config_rx_demo.borrow().clone();

            if !current_config.enable_activation_demo {
                tracing::debug!("Activation demo disabled, waiting for config change");
                if config_rx_demo.changed().await.is_err() {
                    break;
                }
                continue;
            }

            tracing::debug!("Activation demo enabled, running demo cycle");

            activation_demo.wake_via_wake_word().await;
            tracing::info!("Demo: triggered wake word");

            let current_timeout = config_rx_demo.borrow().auto_sleep_timeout_secs;

            let sleep_duration = current_timeout + DEMO_GRACE_PERIOD_SECS;
            tracing::info!(
                "Demo: sleeping for {}s (timeout: {}s + grace: {}s)",
                sleep_duration,
                current_timeout,
                DEMO_GRACE_PERIOD_SECS
            );

            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::from_secs(sleep_duration)) => {
                }
                result = config_rx_demo.changed() => {
                    if result.is_err() {
                        break;
                    }
                    tracing::debug!("Demo: config changed during sleep, restarting cycle");
                }
            }
        }
        tracing::info!("Activation demo task exiting");
    });

    let overlay = Arc::new(OverlayManager::new_with_wayland(&manager, &activation));
    tracing::info!("OverlayManager initialized and running");

    let overlay_monitor = overlay.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        let mut previous_error = false;

        loop {
            interval.tick().await;

            let has_error = overlay_monitor.has_error().await;

            if has_error && !previous_error {
                tracing::warn!("Overlay connection error - attempting reconnection");
            } else if !has_error && previous_error {
                tracing::info!("Overlay connection restored");
            }

            if has_error {
                let status = overlay_monitor.reconnection_status().await;
                if status.ready_to_retry {
                    tracing::warn!(
                        "Overlay reconnecting in {}s (attempt {})",
                        status.next_backoff_duration.as_secs(),
                        status.attempt_count
                    );
                }
            }

            previous_error = has_error;
        }
    });

    println!("phonesc starting with config:");
    println!("  Auto-sleep timeout: {}s", config.auto_sleep_timeout_secs);
    println!(
        "  Command pause: {}ms",
        config.command_pause_threshold_ms
    );
    println!(
        "  Dictation pause: {}ms",
        config.dictation_pause_threshold_ms
    );
    println!("  Overlay position: {}", config.overlay.position);
    println!("  Dictation service: {}", config.dictation_service.url());
    if config.enable_activation_demo {
        println!("  Activation demo mode: ENABLED (cycling every ~{}s)", config.auto_sleep_timeout_secs + 3);
    }

    let mut config_rx = manager.subscribe();
    let activation_for_config = activation.clone();
    tokio::spawn(async move {
        loop {
            if config_rx.changed().await.is_ok() {
                let config = config_rx.borrow().clone();
                tracing::info!("Config updated!");
                tracing::info!("  Auto-sleep timeout: {}s", config.auto_sleep_timeout_secs);
                tracing::info!("  Command pause: {}ms", config.command_pause_threshold_ms);
                tracing::info!("  Dictation pause: {}ms", config.dictation_pause_threshold_ms);
                tracing::info!("  Overlay position: {}", config.overlay.position);
                tracing::info!("  Dictation service: {}", config.dictation_service.url());

                let new_timeout = std::time::Duration::from_secs(config.auto_sleep_timeout_secs);
                activation_for_config.set_timeout(new_timeout).await;
                tracing::info!(
                    "Updated ActivationManager timeout to: {}s",
                    config.auto_sleep_timeout_secs
                );
            } else {
                break;
            }
        }
    });

    let mut state_rx = activation.subscribe();
    tokio::spawn(async move {
        while state_rx.changed().await.is_ok() {
            let (state, transition) = *state_rx.borrow();
            tracing::info!("Activation state changed to: {:?} (via {:?})", state, transition);
        }
    });

    println!("\nLive configuration reload is active.");
    println!("Edit ~/.config/phonesc/config.toml to see changes take effect.");
    println!("Press CTRL+C to exit.\n");
    println!("Activation state manager is running.\n");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received CTRL+C, shutting down...");
        }
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
