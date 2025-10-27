use super::OverlayColor;

/// Default color for awake state (green)
pub const DEFAULT_AWAKE_COLOR: OverlayColor = OverlayColor::opaque(0, 255, 0);

/// Default color for asleep state (gray)
pub const DEFAULT_ASLEEP_COLOR: OverlayColor = OverlayColor::opaque(128, 128, 128);

/// Default color for error state (red)
pub const DEFAULT_ERROR_COLOR: OverlayColor = OverlayColor::opaque(255, 0, 0);

/// Default color name for awake state (used in config file)
pub const DEFAULT_AWAKE_COLOR_NAME: &str = "green";

/// Default color name for asleep state (used in config file)
pub const DEFAULT_ASLEEP_COLOR_NAME: &str = "gray";

/// Default color name for error state (used in config file)
pub const DEFAULT_ERROR_COLOR_NAME: &str = "red";
