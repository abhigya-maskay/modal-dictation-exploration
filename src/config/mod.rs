use std::path::PathBuf;

mod manager;
pub use manager::{ConfigManager, WatcherHealth};

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Config directory not found")]
    DirectoryNotFound,
}

/// Main application configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_auto_sleep_timeout")]
    pub auto_sleep_timeout_secs: u64,

    #[serde(default = "default_command_pause_threshold")]
    pub command_pause_threshold_ms: u64,

    #[serde(default = "default_dictation_pause_threshold")]
    pub dictation_pause_threshold_ms: u64,

    #[serde(default)]
    pub overlay: OverlayConfig,

    #[serde(default)]
    pub dictation_service: DictationServiceConfig,

    #[serde(default = "default_enable_activation_demo")]
    pub enable_activation_demo: bool,

    #[serde(default = "default_activation_demo_interval_secs")]
    pub activation_demo_interval_secs: u64,
}

/// Overlay indicator configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OverlayConfig {
    #[serde(default = "default_overlay_awake_color")]
    pub awake_color: String,

    #[serde(default = "default_overlay_asleep_color")]
    pub asleep_color: String,

    #[serde(default = "default_overlay_error_color")]
    pub error_color: String,

    #[serde(default = "default_overlay_position")]
    pub position: String,
}

/// Dictation service configuration
#[derive(Debug, Clone, serde::Deserialize)]
pub struct DictationServiceConfig {
    #[serde(default = "default_dictation_host")]
    pub host: String,

    #[serde(default = "default_dictation_port")]
    pub port: u16,
}

fn default_auto_sleep_timeout() -> u64 {
    300
}

fn default_command_pause_threshold() -> u64 {
    700
}

fn default_dictation_pause_threshold() -> u64 {
    900
}

fn default_overlay_awake_color() -> String {
    crate::overlay::DEFAULT_AWAKE_COLOR_NAME.to_string()
}

fn default_overlay_asleep_color() -> String {
    crate::overlay::DEFAULT_ASLEEP_COLOR_NAME.to_string()
}

fn default_overlay_error_color() -> String {
    crate::overlay::DEFAULT_ERROR_COLOR_NAME.to_string()
}

fn default_overlay_position() -> String {
    "top-right".to_string()
}

fn default_dictation_host() -> String {
    "127.0.0.1".to_string()
}

fn default_dictation_port() -> u16 {
    5123
}

fn default_enable_activation_demo() -> bool {
    false
}

fn default_activation_demo_interval_secs() -> u64 {
    10
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            awake_color: default_overlay_awake_color(),
            asleep_color: default_overlay_asleep_color(),
            error_color: default_overlay_error_color(),
            position: default_overlay_position(),
        }
    }
}

impl Default for DictationServiceConfig {
    fn default() -> Self {
        Self {
            host: default_dictation_host(),
            port: default_dictation_port(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_sleep_timeout_secs: default_auto_sleep_timeout(),
            command_pause_threshold_ms: default_command_pause_threshold(),
            dictation_pause_threshold_ms: default_dictation_pause_threshold(),
            overlay: OverlayConfig::default(),
            dictation_service: DictationServiceConfig::default(),
            enable_activation_demo: default_enable_activation_demo(),
            activation_demo_interval_secs: default_activation_demo_interval_secs(),
        }
    }
}

impl DictationServiceConfig {
    /// Returns the full HTTP URL for the dictation service
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Config {
    /// Load configuration from a specific path
    ///
    /// Falls back to defaults if the file doesn't exist or cannot be parsed.
    /// Logs errors but does not crash the application.
    pub fn load_from_path(config_path: PathBuf) -> Self {
        if !config_path.exists() {
            tracing::info!("Config file not found at {}, using defaults", config_path.display());
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(config) => {
                    tracing::info!("Successfully loaded config from {}", config_path.display());
                    config
                }
                Err(e) => {
                    tracing::warn!("Failed to parse config: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config file: {}, using defaults", e);
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.auto_sleep_timeout_secs, 300);
        assert_eq!(config.command_pause_threshold_ms, 700);
        assert_eq!(config.dictation_pause_threshold_ms, 900);
        assert_eq!(config.overlay.awake_color, "green");
        assert_eq!(config.overlay.asleep_color, "gray");
        assert_eq!(config.overlay.error_color, "red");
        assert_eq!(config.overlay.position, "top-right");
        assert_eq!(config.dictation_service.host, "127.0.0.1");
        assert_eq!(config.dictation_service.port, 5123);
    }

    #[test]
    fn test_parse_empty_config() {
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auto_sleep_timeout_secs, 300);
        assert_eq!(config.command_pause_threshold_ms, 700);
        assert_eq!(config.dictation_pause_threshold_ms, 900);
    }

    #[test]
    fn test_parse_partial_config() {
        let toml_str = r#"
            auto_sleep_timeout_secs = 600

            [overlay]
            awake_color = "blue"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auto_sleep_timeout_secs, 600);
        assert_eq!(config.command_pause_threshold_ms, 700);
        assert_eq!(config.overlay.awake_color, "blue");
        assert_eq!(config.overlay.asleep_color, "gray");
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
            auto_sleep_timeout_secs = 600
            command_pause_threshold_ms = 800
            dictation_pause_threshold_ms = 1000

            [overlay]
            awake_color = "blue"
            asleep_color = "white"
            error_color = "orange"
            position = "bottom-left"

            [dictation_service]
            host = "192.168.1.100"
            port = 8080
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auto_sleep_timeout_secs, 600);
        assert_eq!(config.command_pause_threshold_ms, 800);
        assert_eq!(config.dictation_pause_threshold_ms, 1000);
        assert_eq!(config.overlay.awake_color, "blue");
        assert_eq!(config.overlay.asleep_color, "white");
        assert_eq!(config.overlay.error_color, "orange");
        assert_eq!(config.overlay.position, "bottom-left");
        assert_eq!(config.dictation_service.host, "192.168.1.100");
        assert_eq!(config.dictation_service.port, 8080);
        assert_eq!(
            config.dictation_service.url(),
            "http://192.168.1.100:8080"
        );
    }

    #[test]
    fn test_invalid_toml() {
        let toml_str = "invalid { toml";
        let result = toml::from_str::<Config>(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_dictation_service_url_formatting() {
        let service = DictationServiceConfig {
            host: "127.0.0.1".to_string(),
            port: 9000,
        };
        assert_eq!(service.url(), "http://127.0.0.1:9000");
    }

    #[test]
    fn test_default_overlay_config() {
        let overlay = OverlayConfig::default();
        assert_eq!(overlay.awake_color, "green");
        assert_eq!(overlay.asleep_color, "gray");
        assert_eq!(overlay.error_color, "red");
        assert_eq!(overlay.position, "top-right");
    }

    #[test]
    fn test_default_dictation_service_config() {
        let service = DictationServiceConfig::default();
        assert_eq!(service.host, "127.0.0.1");
        assert_eq!(service.port, 5123);
        assert_eq!(service.url(), "http://127.0.0.1:5123");
    }

    #[test]
    fn test_demo_config_defaults() {
        let config = Config::default();
        assert_eq!(config.enable_activation_demo, false);
        assert_eq!(config.activation_demo_interval_secs, 10);
    }

    #[test]
    fn test_parse_demo_config_enabled() {
        let toml_str = r#"
            enable_activation_demo = true
            activation_demo_interval_secs = 5
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.enable_activation_demo, true);
        assert_eq!(config.activation_demo_interval_secs, 5);
    }

    #[test]
    fn test_parse_demo_config_disabled() {
        let toml_str = r#"
            enable_activation_demo = false
            activation_demo_interval_secs = 15
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.enable_activation_demo, false);
        assert_eq!(config.activation_demo_interval_secs, 15);
    }

    #[test]
    fn test_parse_config_with_demo_and_other_settings() {
        let toml_str = r#"
            auto_sleep_timeout_secs = 600
            enable_activation_demo = true
            activation_demo_interval_secs = 8

            [overlay]
            awake_color = "blue"
            asleep_color = "white"

            [dictation_service]
            host = "192.168.1.100"
            port = 8080
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.auto_sleep_timeout_secs, 600);
        assert_eq!(config.enable_activation_demo, true);
        assert_eq!(config.activation_demo_interval_secs, 8);
        assert_eq!(config.overlay.awake_color, "blue");
        assert_eq!(config.dictation_service.host, "192.168.1.100");
    }
}
