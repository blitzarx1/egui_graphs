// Shared margins
pub const UI_MARGIN: f32 = 10.0;

// Common spacing between grouped controls
pub const SECTION_SPACING: f32 = 6.0;

// Side panel sizing
pub const SIDE_PANEL_WIDTH: f32 = 300.0;

// Overlay buttons (bottom-right)
pub const OVERLAY_BTN_SIZE: f32 = 28.0; // square buttons: width=height
pub const OVERLAY_BTN_SPACING: f32 = 6.0;
pub const OVERLAY_ICON_SIZE: f32 = 16.0;

// Text sizes
pub const INFO_TEXT_SIZE: f32 = 11.0; // bottom info overlay
pub const DEBUG_MONO_FONT_SIZE: f32 = 14.0; // debug overlay monospace
pub const HEADING_TEXT_SIZE: f32 = 16.0; // headings / prominent labels

// Sections specific
pub const SELECTED_SCROLL_MAX_HEIGHT: f32 = 150.0;
#[cfg(feature = "events")]
pub const EVENTS_MIN_HEIGHT: f32 = 220.0;
