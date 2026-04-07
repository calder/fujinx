use std::fmt;
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use yansi::{Paint, Style};

use crate::ptp::{CameraInfo, ObjectInfo, Session, UsbDevice};
use crate::{BLUE, RED, SUPPORTED_MODELS, YELLOW};

use super::codec::{
    dr_decode, dr_encode, encode_recipe, nr_decode, nr_encode, wb_decode, wb_encode,
};
use super::recipe::Recipe;
use super::{format, prop};

/// Fuji vendor PTP operation codes.
const FUJI_OC_SEND_OBJECT_INFO: u16 = 0x900C;
const FUJI_OC_SEND_OBJECT: u16 = 0x900D;

/// Opaque conversion profile read from the camera for a loaded RAF.
/// Pass to [`Camera::render`] to apply a recipe.
pub struct Profile(Vec<u8>);

/// A connected camera.
pub struct Camera {
    /// PTP session.
    session: Session,

    /// Camera info.
    info: CameraInfo,

    /// Whether this camera model is officially supported.
    supported: bool,
}

impl Camera {
    /// Open the first camera found.
    pub fn open_first() -> Result<Self> {
        let detected = crate::ptp::detect()?;
        let camera = detected
            .first()
            .ok_or_else(|| anyhow!("no cameras found"))?;

        Self::open(camera)
    }

    /// Open a camera.
    fn open(camera: &UsbDevice) -> Result<Self> {
        let mut session = Session::open(camera)?;
        let info = session.get_camera_info()?;
        let supported = SUPPORTED_MODELS.contains(&info.model.as_str());
        let cam = Self {
            session,
            info,
            supported,
        };

        Ok(cam)
    }

    /// Get camera info.
    pub fn info(&self) -> &CameraInfo {
        &self.info
    }

    /// Upload a RAF and read its conversion profile from the camera.
    ///
    /// Call this once per RAF, then call [`render`] for each recipe.
    pub fn load_raw(&mut self, raw: &[u8]) -> Result<Profile> {
        let info = ObjectInfo {
            format_code: format::FUJI_UPLOAD,
            compressed_size: raw.len() as u32,
            filename: "FUP_FILE.dat".to_string(),
        };
        self.session.vendor_execute(
            FUJI_OC_SEND_OBJECT_INFO,
            &[0, 0, 0],
            Some(&info.to_dataset()),
        )?;
        self.session
            .vendor_execute(FUJI_OC_SEND_OBJECT, &[], Some(raw))?;
        let data = self
            .session
            .get_device_prop_value_raw(prop::RAW_CONVERSION_PROFILE)?;

        Ok(Profile(data))
    }

    /// Apply a recipe to a loaded profile and convert to JPEG.
    pub fn render(&mut self, profile: &Profile, recipe: &Recipe) -> Result<Vec<u8>> {
        let mut data = profile.0.clone();
        encode_recipe(recipe, &mut data);
        self.session
            .set_device_prop_value_raw(prop::RAW_CONVERSION_PROFILE, &data)?;

        self.convert_raw()
    }

    /// Trigger RAW conversion and wait for the JPEG result.
    fn convert_raw(&mut self) -> Result<Vec<u8>> {
        // Start conversion. 0=half-res, 1=full-res.
        self.session
            .set_device_prop_value_u16(prop::START_RAW_CONVERSION, 1)?;

        // Poll for the converted JPEG.
        for _ in 0..30 {
            std::thread::sleep(Duration::from_secs(1));

            let handles = self.session.get_object_handles(0xFFFFFFFF, 0, 0)?;
            if !handles.is_empty() {
                let jpeg = self.session.get_object(handles[0])?;
                let _ = self.session.delete_object(handles[0]);

                return Ok(jpeg);
            }
        }

        bail!("timed out waiting for converted JPEG")
    }

    /// Read a custom preset slot (1–7) and return its recipe.
    pub fn read_preset(&mut self, slot: u8) -> Result<Recipe> {
        self.session
            .set_device_prop_value_u16(prop::PRESET_SLOT, slot as u16)?;

        let name = self
            .session
            .get_device_prop_value_string(prop::PRESET_NAME)?;

        let mut prop_i16 = |code| {
            self.session
                .get_device_prop_value_i16(code)
                .map(|v| v as i32)
        };

        let wb_mode = prop_i16(prop::PRESET_WHITE_BALANCE)? & 0xFFFF;
        let wb_temp = prop_i16(prop::PRESET_WB_COLOR_TEMP)?;

        Ok(Recipe {
            name,
            film: prop_i16(prop::PRESET_FILM_SIMULATION)?.try_into()?,
            grain: prop_i16(prop::PRESET_GRAIN_EFFECT)?.try_into()?,
            color_chrome: prop_i16(prop::PRESET_COLOR_CHROME)?.try_into()?,
            color_chrome_blue: prop_i16(prop::PRESET_COLOR_CHROME_BLUE)?.try_into()?,
            white_balance: wb_decode(wb_mode, wb_temp)?,
            white_balance_red: prop_i16(prop::PRESET_WB_SHIFT_R)?,
            white_balance_blue: prop_i16(prop::PRESET_WB_SHIFT_B)?,
            dynamic_range: dr_decode(prop_i16(prop::PRESET_DYNAMIC_RANGE)?),
            dynamic_range_priority: super::recipe::DynamicRangePriority::Off,
            exposure: 0.0,
            highlight: prop_i16(prop::PRESET_HIGHLIGHT_TONE)? as f64 / 10.0,
            shadow: prop_i16(prop::PRESET_SHADOW_TONE)? as f64 / 10.0,
            color: prop_i16(prop::PRESET_COLOR)? as f64 / 10.0,
            sharpness: prop_i16(prop::PRESET_SHARPNESS)? as f64 / 10.0,
            high_iso_nr: nr_decode(prop_i16(prop::PRESET_HIGH_ISO_NR)?),
            clarity: prop_i16(prop::PRESET_CLARITY)? / 10,
        })
    }

    /// Write a recipe into a custom preset slot (1–7).
    pub fn write_preset(&mut self, slot: u8, recipe: &Recipe) -> Result<()> {
        self.session
            .set_device_prop_value_u16(prop::PRESET_SLOT, slot as u16)?;

        self.session
            .set_device_prop_value_string(prop::PRESET_NAME, &recipe.name)?;

        let mut set_i16 =
            |code, value: i32| self.session.set_device_prop_value_i16(code, value as i16);

        set_i16(prop::PRESET_FILM_SIMULATION, recipe.film as i32)?;
        set_i16(prop::PRESET_GRAIN_EFFECT, recipe.grain as i32)?;
        set_i16(prop::PRESET_COLOR_CHROME, recipe.color_chrome as i32)?;
        set_i16(
            prop::PRESET_COLOR_CHROME_BLUE,
            recipe.color_chrome_blue as i32,
        )?;

        let (wb_mode, wb_temp) = wb_encode(&recipe.white_balance);
        set_i16(prop::PRESET_WHITE_BALANCE, wb_mode)?;
        if let Some(t) = wb_temp {
            set_i16(prop::PRESET_WB_COLOR_TEMP, t)?;
        }
        set_i16(prop::PRESET_WB_SHIFT_R, recipe.white_balance_red)?;
        set_i16(prop::PRESET_WB_SHIFT_B, recipe.white_balance_blue)?;
        set_i16(prop::PRESET_DYNAMIC_RANGE, dr_encode(&recipe.dynamic_range))?;
        set_i16(
            prop::PRESET_HIGHLIGHT_TONE,
            (recipe.highlight * 10.0).round() as i32,
        )?;
        set_i16(
            prop::PRESET_SHADOW_TONE,
            (recipe.shadow * 10.0).round() as i32,
        )?;
        set_i16(prop::PRESET_COLOR, (recipe.color * 10.0).round() as i32)?;
        set_i16(
            prop::PRESET_SHARPNESS,
            (recipe.sharpness * 10.0).round() as i32,
        )?;
        set_i16(prop::PRESET_HIGH_ISO_NR, nr_encode(recipe.high_iso_nr))?;
        set_i16(prop::PRESET_CLARITY, recipe.clarity * 10)?;

        Ok(())
    }

    /// Get all connected Fujifilm cameras whether or not they're suported.
    pub fn detect() -> Result<Vec<Camera>> {
        crate::ptp::detect()?.iter().map(Camera::open).collect()
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            "[".paint(RED),
            self.info.model.paint(BLUE).bold(),
            match self.supported {
                true => "".paint(Style::new()),
                false => " UNSUPPORTED".paint(YELLOW),
            },
            "]".paint(RED),
        )
    }
}
