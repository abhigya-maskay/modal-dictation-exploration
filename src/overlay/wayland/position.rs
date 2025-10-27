use smithay_client_toolkit::shell::wlr_layer::Anchor;

/// Parses overlay position string into anchor values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl OverlayPosition {
    /// Parses a position string (e.g., "top-right", "bottom-left")
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.trim().to_lowercase().as_str() {
            "top-left" => Ok(OverlayPosition::TopLeft),
            "top-right" => Ok(OverlayPosition::TopRight),
            "bottom-left" => Ok(OverlayPosition::BottomLeft),
            "bottom-right" => Ok(OverlayPosition::BottomRight),
            _ => Err(format!(
                "Invalid position: {}. Use: top-left, top-right, bottom-left, or bottom-right",
                s
            )),
        }
    }

    /// Returns the anchor values as Anchor bitflags for layer shell protocol
    pub fn anchor_flags(self) -> Anchor {
        match self {
            OverlayPosition::TopLeft => Anchor::TOP | Anchor::LEFT,
            OverlayPosition::TopRight => Anchor::TOP | Anchor::RIGHT,
            OverlayPosition::BottomLeft => Anchor::BOTTOM | Anchor::LEFT,
            OverlayPosition::BottomRight => Anchor::BOTTOM | Anchor::RIGHT,
        }
    }

    /// Returns the canonical string representation of this position
    pub fn as_str(&self) -> &'static str {
        match self {
            OverlayPosition::TopLeft => "top-left",
            OverlayPosition::TopRight => "top-right",
            OverlayPosition::BottomLeft => "bottom-left",
            OverlayPosition::BottomRight => "bottom-right",
        }
    }
}

impl std::fmt::Display for OverlayPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_position_parsing() {
        assert_eq!(
            OverlayPosition::from_str("top-right").unwrap(),
            OverlayPosition::TopRight
        );
        assert_eq!(
            OverlayPosition::from_str("top-left").unwrap(),
            OverlayPosition::TopLeft
        );
        assert_eq!(
            OverlayPosition::from_str("bottom-right").unwrap(),
            OverlayPosition::BottomRight
        );
        assert_eq!(
            OverlayPosition::from_str("bottom-left").unwrap(),
            OverlayPosition::BottomLeft
        );
    }

    #[test]
    fn test_overlay_position_case_insensitive() {
        assert_eq!(
            OverlayPosition::from_str("TOP-RIGHT").unwrap(),
            OverlayPosition::TopRight
        );
        assert_eq!(
            OverlayPosition::from_str("Bottom-Left").unwrap(),
            OverlayPosition::BottomLeft
        );
    }

    #[test]
    fn test_overlay_position_invalid() {
        assert!(OverlayPosition::from_str("invalid").is_err());
        assert!(OverlayPosition::from_str("center").is_err());
    }
}
