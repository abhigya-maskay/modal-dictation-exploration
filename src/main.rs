mod activation;
mod config;

use std::sync::Arc;
use activation::ActivationManager;
use config::ConfigManager;

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

    let activation_demo = activation.clone();
    tokio::spawn(async move {
        use std::time::Duration;
        tokio::time::sleep(Duration::from_secs(1)).await;

        tracing::info!("DEMO: Simulating wake word detection...");
        activation_demo.wake_via_wake_word().await;

        tokio::time::sleep(Duration::from_millis(500)).await;
        tracing::info!("DEMO: Simulating command activity...");
        activation_demo.on_command_activity().await;

        tokio::time::sleep(Duration::from_millis(500)).await;
        tracing::info!("DEMO: Simulating sleep command...");
        activation_demo.sleep_via_command().await;

        tracing::info!("DEMO: Finished");
    });

    println!("\nLive configuration reload is active.");
    println!("Edit ~/.config/phonesc/config.toml to see changes take effect.");
    println!("Press CTRL+C to exit.\n");
    println!("Activation state manager is running (demo shims active).\n");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received CTRL+C, shutting down...");
        }
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
