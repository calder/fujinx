use crate::Result;

use super::dataset::DatasetReader;

/// Device information from a connected camera.
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub manufacturer: String,
    pub model: String,
    pub device_version: String,
    pub serial_number: String,
    pub operations_supported: Vec<u16>,
    pub device_properties_supported: Vec<u16>,
    pub capture_formats: Vec<u16>,
    pub image_formats: Vec<u16>,
}

impl CameraInfo {
    /// Parse PTP DeviceInfo dataset (PTP spec 5.5.1).
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = DatasetReader::new(data);

        // Skip: StandardVersion(u16) + VendorExtensionID(u32) + VendorExtensionVersion(u16)
        r.skip(8)?;
        r.read_ptp_string()?; // VendorExtensionDesc
        r.skip(2)?; // FunctionalMode
        let operations_supported = r.read_u16_array()?;
        r.skip_u16_array()?; // Events
        let device_properties_supported = r.read_u16_array()?;
        let capture_formats = r.read_u16_array()?;
        let image_formats = r.read_u16_array()?;
        let manufacturer = r.read_ptp_string()?;
        let model = r.read_ptp_string()?;
        let device_version = r.read_ptp_string()?;
        let serial_number = r.read_ptp_string()?;

        Ok(Self {
            manufacturer,
            model,
            device_version,
            serial_number,
            operations_supported,
            device_properties_supported,
            capture_formats,
            image_formats,
        })
    }
}
