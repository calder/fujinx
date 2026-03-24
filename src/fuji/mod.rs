mod camera;
mod codec;
pub(crate) mod recipe;

pub use camera::{Camera, Profile};
pub use recipe::Recipe;

/// PTP object format codes.
pub(crate) mod format {
    /// Format code used when uploading RAF files for conversion.
    pub const FUJI_UPLOAD: u16 = 0xF802;
}

/// Fujifilm vendor PTP device property codes.
pub(crate) mod prop {
    /// RAW conversion profile.
    pub const RAW_CONVERSION_PROFILE: u16 = 0xD185;

    /// Write to trigger RAW conversion. Value = quality: 0 = preview (half-res), 1 = full-res.
    pub const START_RAW_CONVERSION: u16 = 0xD183;

    /// Preset slot selector (write slot number to select, then read D18D–D1A5).
    pub const PRESET_SLOT: u16 = 0xD18C;

    /// Preset name (PTP string).
    pub const PRESET_NAME: u16 = 0xD18D;

    /// Preset properties (see `Recipe`).
    pub const PRESET_DYNAMIC_RANGE: u16 = 0xD190;

    /// Preset film simulation (see `Recipe`).
    pub const PRESET_FILM_SIMULATION: u16 = 0xD192;

    /// Preset grain effect (see `Recipe`).
    pub const PRESET_GRAIN_EFFECT: u16 = 0xD195;

    /// Preset color chrome (see `Recipe`).
    pub const PRESET_COLOR_CHROME: u16 = 0xD196;

    /// Preset color chrome blue (see `Recipe`).
    pub const PRESET_COLOR_CHROME_BLUE: u16 = 0xD197;

    /// Preset white balance (see `Recipe`).
    pub const PRESET_WHITE_BALANCE: u16 = 0xD199;

    /// Preset white balance red shift (see `Recipe`).
    pub const PRESET_WB_SHIFT_R: u16 = 0xD19A;

    /// Preset white balance blue shift (see `Recipe`).
    pub const PRESET_WB_SHIFT_B: u16 = 0xD19B;

    /// Preset white balance color temperature (see `Recipe`).
    pub const PRESET_WB_COLOR_TEMP: u16 = 0xD19C;

    /// Preset highlight tone (see `Recipe`).
    pub const PRESET_HIGHLIGHT_TONE: u16 = 0xD19D;

    /// Preset shadow tone (see `Recipe`).
    pub const PRESET_SHADOW_TONE: u16 = 0xD19E;

    /// Preset color shift (see `Recipe`).
    pub const PRESET_COLOR: u16 = 0xD19F;

    /// Preset sharpness shift (see `Recipe`).
    pub const PRESET_SHARPNESS: u16 = 0xD1A0;

    /// Preset high ISO noise reduction (see `Recipe`).
    pub const PRESET_HIGH_ISO_NR: u16 = 0xD1A1;

    /// Preset clarity (see `Recipe`).
    pub const PRESET_CLARITY: u16 = 0xD1A2;
}
