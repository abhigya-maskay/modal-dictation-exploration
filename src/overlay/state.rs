use crate::activation::SystemState;
use crate::config::OverlayConfig;
use crate::overlay::renderer::{parse_color, parse_color_with_fallback, OverlayColor, RendererError};
use std::time::{Duration, SystemTime};

/// Represents the overlay's current rendering state
#[derive(Debug, Clone)]
pub struct OverlayRenderState {
    /// Current system state
    pub system_state: SystemState,
    /// Whether an error is present
    pub has_error: bool,
    /// Current configuration
    pub config: OverlayConfig,
    /// Parsed colors from config
    pub awake_color: OverlayColor,
    pub asleep_color: OverlayColor,
    pub error_color: OverlayColor,
}

impl OverlayRenderState {
    /// Creates a new overlay render state from config and system state
    ///
    /// Uses fallback colors if any color in the config cannot be parsed, ensuring
    /// the overlay can always be initialized. Invalid colors are logged as warnings
    /// and will be correctable via live config reload.
    pub fn new(
        system_state: SystemState,
        config: OverlayConfig,
    ) -> Self {
        let awake_color = parse_color_with_fallback(&config.awake_color, OverlayColor::opaque(0, 255, 0));
        let asleep_color = parse_color_with_fallback(&config.asleep_color, OverlayColor::opaque(128, 128, 128));
        let error_color = parse_color_with_fallback(&config.error_color, OverlayColor::opaque(255, 0, 0));

        Self {
            system_state,
            has_error: false,
            config,
            awake_color,
            asleep_color,
            error_color,
        }
    }

    /// Updates the system state
    pub fn update_system_state(&mut self, new_state: SystemState) {
        self.system_state = new_state;
    }

    /// Sets or clears the error state
    pub fn set_error(&mut self, has_error: bool) {
        self.has_error = has_error;
    }

    /// Updates the configuration and re-parses colors
    pub fn update_config(&mut self, new_config: OverlayConfig) -> Result<(), RendererError> {
        let awake_color = parse_color(&new_config.awake_color)?;
        let asleep_color = parse_color(&new_config.asleep_color)?;
        let error_color = parse_color(&new_config.error_color)?;

        self.config = new_config;
        self.awake_color = awake_color;
        self.asleep_color = asleep_color;
        self.error_color = error_color;

        Ok(())
    }

    /// Returns the current color based on system state and error flag
    pub fn current_color(&self) -> OverlayColor {
        crate::overlay::renderer::state_to_color(
            self.system_state,
            self.awake_color,
            self.asleep_color,
            self.error_color,
            self.has_error,
        )
    }
}

/// Tracks reconnection attempts with exponential backoff
pub struct ReconnectionState {
    /// Number of failed attempts
    pub attempt_count: u32,
    /// Time of last attempt
    pub last_attempt: SystemTime,
}

impl ReconnectionState {
    /// Creates a new reconnection state
    pub fn new() -> Self {
        Self {
            attempt_count: 0,
            last_attempt: SystemTime::now(),
        }
    }

    /// Calculates the backoff duration for the next attempt
    ///
    /// Uses exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s (capped)
    pub fn next_backoff(&self) -> Duration {
        let base_millis = 1000; // 1 second base
        let exponent = std::cmp::min(std::cmp::max(self.attempt_count as i32 - 1, 0) as u32, 5); // Cap at 2^5 = 32s base
        let millis = base_millis * 2_u64.pow(exponent);
        let capped = std::cmp::min(millis, 30000); // Cap at 30 seconds
        Duration::from_millis(capped)
    }

    /// Records a failed attempt and returns time to wait before retry
    pub fn record_failure(&mut self) -> Duration {
        self.attempt_count += 1;
        self.last_attempt = SystemTime::now();
        self.next_backoff()
    }

    /// Resets the backoff state on successful connection
    pub fn reset(&mut self) {
        self.attempt_count = 0;
        self.last_attempt = SystemTime::now();
    }

    /// Returns whether it's time to attempt reconnection
    pub fn should_retry(&self) -> bool {
        let elapsed = self
            .last_attempt
            .elapsed()
            .unwrap_or(Duration::from_secs(1));
        elapsed >= self.next_backoff()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnection_exponential_backoff() {
        let mut state = ReconnectionState::new();

        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 1000);
        assert_eq!(state.attempt_count, 1);

        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 2000);
        assert_eq!(state.attempt_count, 2);

        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 4000);
        assert_eq!(state.attempt_count, 3);

        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 8000);
        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 16000);
        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 30000);

        let wait = state.record_failure();
        assert_eq!(wait.as_millis(), 30000);
    }

    #[test]
    fn test_reconnection_reset() {
        let mut state = ReconnectionState::new();

        state.record_failure();
        state.record_failure();
        assert_eq!(state.attempt_count, 2);

        state.reset();
        assert_eq!(state.attempt_count, 0);
    }

    #[test]
    fn test_overlay_render_state_creation() {
        let config = crate::config::OverlayConfig::default();
        let state = OverlayRenderState::new(SystemState::Awake, config);

        assert_eq!(state.system_state, SystemState::Awake);
        assert!(!state.has_error);
    }

    #[test]
    fn test_overlay_render_state_color_selection() {
        let config = crate::config::OverlayConfig::default();
        let mut state = OverlayRenderState::new(SystemState::Awake, config);

        let color = state.current_color();
        assert_eq!(color, crate::overlay::renderer::OverlayColor::opaque(0, 255, 0));

        state.update_system_state(SystemState::Asleep);
        let color = state.current_color();
        assert_eq!(color, crate::overlay::renderer::OverlayColor::opaque(128, 128, 128));

        state.update_system_state(SystemState::Awake);
        state.set_error(true);
        let color = state.current_color();
        assert_eq!(color, crate::overlay::renderer::OverlayColor::opaque(255, 0, 0));
    }

    #[test]
    fn test_overlay_config_update() {
        let mut config = crate::config::OverlayConfig::default();
        config.awake_color = "blue".to_string();

        let mut state = OverlayRenderState::new(SystemState::Awake, config);

        let color = state.current_color();
        assert_eq!(color, crate::overlay::renderer::OverlayColor::opaque(0, 0, 255));

        let mut new_config = crate::config::OverlayConfig::default();
        new_config.awake_color = "green".to_string();
        state.update_config(new_config).expect("Failed to update config");

        let color = state.current_color();
        assert_eq!(color, crate::overlay::renderer::OverlayColor::opaque(0, 255, 0));
    }

    #[test]
    fn test_overlay_fallback_colors_on_invalid_config() {
        let mut config = crate::config::OverlayConfig::default();
        config.awake_color = "invalid_color".to_string();
        config.asleep_color = "not_a_color".to_string();
        config.error_color = "badcolor".to_string();

        let state = OverlayRenderState::new(SystemState::Awake, config);

        assert_eq!(state.awake_color, crate::overlay::renderer::OverlayColor::opaque(0, 255, 0));
        assert_eq!(state.asleep_color, crate::overlay::renderer::OverlayColor::opaque(128, 128, 128));
        assert_eq!(state.error_color, crate::overlay::renderer::OverlayColor::opaque(255, 0, 0));
    }
}
