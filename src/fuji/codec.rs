use anyhow::{Result, bail};

use super::recipe::{DynamicRange, Recipe, WhiteBalance};

/// Parameter indices within the binary conversion profile.
pub(super) mod param_idx {
    pub const EXPOSURE_BIAS: usize = 4;
    pub const DYNAMIC_RANGE: usize = 6;
    pub const DYNAMIC_RANGE_PRIORITY: usize = 7;
    pub const FILM_SIMULATION: usize = 8;
    pub const GRAIN_EFFECT: usize = 9;
    pub const COLOR_CHROME: usize = 10;
    pub const WB_SHOOT_COND: usize = 11;
    pub const WHITE_BALANCE: usize = 12;
    pub const WB_SHIFT_R: usize = 13;
    pub const WB_SHIFT_B: usize = 14;
    pub const WB_COLOR_TEMP: usize = 15;
    pub const HIGHLIGHT_TONE: usize = 16;
    pub const SHADOW_TONE: usize = 17;
    pub const COLOR: usize = 18;
    pub const SHARPNESS: usize = 19;
    pub const NOISE_REDUCTION: usize = 20;
    pub const COLOR_CHROME_BLUE: usize = 25;
    pub const CLARITY: usize = 27;
}

/// Noise reduction: proprietary wire encoding (u16 bit pattern) → UI value (-4..+4).
const NR_DECODE: [(i32, i32); 9] = [
    (0x8000, -4),
    (0x7000, -3),
    (0x4000, -2),
    (0x3000, -1),
    (0x2000, 0),
    (0x1000, 1),
    (0x0000, 2),
    (0x6000, 3),
    (0x5000, 4),
];

pub(super) fn nr_decode(raw: i32) -> i32 {
    NR_DECODE
        .iter()
        .find(|(enc, _)| *enc == raw)
        .map(|(_, ui)| *ui)
        .unwrap_or(0)
}

pub(super) fn nr_encode(ui: i32) -> i32 {
    NR_DECODE
        .iter()
        .find(|(_, val)| *val == ui)
        .map(|(enc, _)| *enc)
        .unwrap_or(0x2000)
}

pub(super) fn get_param(data: &[u8], index: usize) -> i32 {
    let num_params = u16::from_le_bytes([data[0], data[1]]) as usize;
    let off = data.len() - num_params * 4 + index * 4;
    i32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}

fn set_param(data: &mut [u8], index: usize, value: i32) {
    let num_params = u16::from_le_bytes([data[0], data[1]]) as usize;
    let off = data.len() - num_params * 4 + index * 4;
    data[off..off + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn wb_encode(wb: &WhiteBalance) -> (i32, Option<i32>) {
    match wb {
        WhiteBalance::Auto => (0x0002, None),
        WhiteBalance::Daylight => (0x0004, None),
        WhiteBalance::Incandescent => (0x0006, None),
        WhiteBalance::Underwater => (0x0008, None),
        WhiteBalance::Fluorescent1 => (0x8001, None),
        WhiteBalance::Fluorescent2 => (0x8002, None),
        WhiteBalance::Fluorescent3 => (0x8003, None),
        WhiteBalance::Shade => (0x8006, None),
        WhiteBalance::Temperature(k) => (0x8007, Some(*k as i32)),
    }
}

pub(super) fn dr_encode(dr: &DynamicRange) -> i32 {
    match dr {
        DynamicRange::DR100 => 100,
        DynamicRange::DR200 => 200,
        DynamicRange::DR400 => 400,
    }
}

pub(super) fn wb_decode(mode: i32, temp: i32) -> Result<WhiteBalance> {
    match mode {
        0 | 0x0002 => Ok(WhiteBalance::Auto),
        0x0004 => Ok(WhiteBalance::Daylight),
        0x0006 => Ok(WhiteBalance::Incandescent),
        0x0008 => Ok(WhiteBalance::Underwater),
        0x8001 => Ok(WhiteBalance::Fluorescent1),
        0x8002 => Ok(WhiteBalance::Fluorescent2),
        0x8003 => Ok(WhiteBalance::Fluorescent3),
        0x8006 => Ok(WhiteBalance::Shade),
        0x8007 => Ok(WhiteBalance::Temperature(temp as u32)),
        _ => bail!("unknown white balance mode: {mode:#06X}"),
    }
}

pub(super) fn dr_decode(raw: i32) -> DynamicRange {
    match raw {
        200 => DynamicRange::DR200,
        400 => DynamicRange::DR400,
        _ => DynamicRange::DR100,
    }
}

pub(super) fn encode_recipe(recipe: &Recipe, data: &mut [u8]) {
    set_param(data, param_idx::FILM_SIMULATION, recipe.film as i32);
    set_param(data, param_idx::GRAIN_EFFECT, recipe.grain as i32);
    set_param(data, param_idx::COLOR_CHROME, recipe.color_chrome as i32);
    set_param(
        data,
        param_idx::COLOR_CHROME_BLUE,
        recipe.color_chrome_blue as i32,
    );
    set_param(data, param_idx::WB_SHOOT_COND, 2); // Override WB
    let (wb_mode, wb_temp) = wb_encode(&recipe.white_balance);
    set_param(data, param_idx::WHITE_BALANCE, wb_mode);
    if let Some(t) = wb_temp {
        set_param(data, param_idx::WB_COLOR_TEMP, t);
    }
    set_param(data, param_idx::WB_SHIFT_R, recipe.white_balance_red);
    set_param(data, param_idx::WB_SHIFT_B, recipe.white_balance_blue);
    set_param(
        data,
        param_idx::DYNAMIC_RANGE,
        dr_encode(&recipe.dynamic_range),
    );
    set_param(
        data,
        param_idx::DYNAMIC_RANGE_PRIORITY,
        recipe.dynamic_range_priority as i32,
    );
    set_param(
        data,
        param_idx::EXPOSURE_BIAS,
        (recipe.exposure * 1000.0).round() as i32,
    );
    set_param(
        data,
        param_idx::HIGHLIGHT_TONE,
        (recipe.highlight * 10.0).round() as i32,
    );
    set_param(
        data,
        param_idx::SHADOW_TONE,
        (recipe.shadow * 10.0).round() as i32,
    );
    set_param(data, param_idx::COLOR, (recipe.color * 10.0).round() as i32);
    set_param(
        data,
        param_idx::SHARPNESS,
        (recipe.sharpness * 10.0).round() as i32,
    );
    set_param(
        data,
        param_idx::NOISE_REDUCTION,
        nr_encode(recipe.high_iso_nr),
    );
    set_param(data, param_idx::CLARITY, recipe.clarity * 10);
}
