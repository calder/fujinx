mod colors;
mod config;
mod fuji;
mod ptp;

pub use crate::colors::{BLUE, GREEN, RED, YELLOW};
pub use crate::config::{Config, RecipeSource};
pub use crate::fuji::{Camera, Profile, Recipe};
pub use crate::ptp::{CameraInfo, ObjectInfo};

/// Supported camera models.
const SUPPORTED_MODELS: &[&str] = &["X-M5"];
