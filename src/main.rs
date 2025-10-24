mod config;

use config::Config;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = Config::load();

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
}
