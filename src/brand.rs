//! Brand-kit foundation for `dev-tools` output (HTML meta-report, etc.).
//!
//! v0.9.3 ships only the seam. The constants here are placeholders; the
//! final palette and footer text land in a later release once the brand
//! kit is finalized. Downstream renderers (the HTML meta-report in
//! particular) reference these constants through CSS custom properties
//! so the same template renders with the real palette once it lands.
//!
//! These constants are not feature-gated — the HTML meta-report depends
//! on them being available in the default build.
//!
//! # Example
//!
//! ```
//! use dev_tools::brand;
//!
//! // Placeholder values; real palette lands later.
//! assert!(brand::COLOR_ACCENT.starts_with('#'));
//! assert!(!brand::FOOTER.is_empty());
//! ```

/// Primary accent color (links, focused borders, hero badges). Placeholder.
pub const COLOR_ACCENT: &str = "#0066cc";

/// Color used to mark a passing verdict. Placeholder.
pub const COLOR_PASS: &str = "#1f8d3a";

/// Color used to mark a failing verdict. Placeholder.
pub const COLOR_FAIL: &str = "#c0392b";

/// Color used to mark a warning verdict. Placeholder.
pub const COLOR_WARN: &str = "#d68910";

/// Color used to mark a lint finding (clippy etc.). Placeholder.
pub const COLOR_LINT: &str = "#d68910";

/// Background color for dark surfaces. Placeholder.
pub const COLOR_BG: &str = "#0e1116";

/// Foreground (body text) color. Placeholder.
pub const COLOR_FG: &str = "#e6e6e6";

/// Footer line printed by the HTML meta-report. Placeholder.
pub const FOOTER: &str = "dev-tools verification suite";
