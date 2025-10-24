mod config;

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
            } else {
                break;
            }
        }
    });

    println!("\nLive configuration reload is active.");
    println!("Edit ~/.config/phonesc/config.toml to see changes take effect.");
    println!("Press CTRL+C to exit.\n");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received CTRL+C, shutting down...");
        }
        Err(err) => {
            tracing::error!("Unable to listen for shutdown signal: {}", err);
        }
    }
}
