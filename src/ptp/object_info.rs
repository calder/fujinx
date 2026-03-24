use crate::Result;

use super::dataset::{DatasetReader, DatasetWriter};

/// PTP ObjectInfo dataset fields.
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub format_code: u16,
    pub compressed_size: u32,
    pub filename: String,
}

impl ObjectInfo {
    /// Parse a PTP ObjectInfo dataset.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = DatasetReader::new(data);

        r.skip(4)?; // StorageID
        let format_code = r.read_u16()?;
        r.skip(2)?; // ProtectionStatus
        let compressed_size = r.read_u32()?;
        r.skip(2 + 4 * 7 + 2)?; // ThumbFormat(u16)..ParentObject(u32) + AssociationType(u16)
        r.skip(4 + 4)?; // AssociationDesc + SequenceNumber
        let filename = r.read_ptp_string()?;

        Ok(Self {
            format_code,
            compressed_size,
            filename,
        })
    }

    /// Build a PTP ObjectInfo dataset (PTP spec 5.5.2).
    pub fn to_dataset(&self) -> Vec<u8> {
        let mut w = DatasetWriter::new();

        w.write_u32(0); // StorageID (ignored on send)
        w.write_u16(self.format_code);
        w.write_u16(0); // ProtectionStatus
        w.write_u32(self.compressed_size);
        w.write_u16(0); // ThumbFormat
        w.write_u32(0); // ThumbCompressedSize
        w.write_u32(0); // ThumbPixWidth
        w.write_u32(0); // ThumbPixHeight
        w.write_u32(0); // ImagePixWidth
        w.write_u32(0); // ImagePixHeight
        w.write_u32(0); // ImageBitDepth
        w.write_u32(0); // ParentObject
        w.write_u16(0); // AssociationType
        w.write_u32(0); // AssociationDesc
        w.write_u32(0); // SequenceNumber
        w.write_ptp_string(&self.filename);
        w.write_ptp_string(""); // CaptureDate
        w.write_ptp_string(""); // ModificationDate
        w.write_ptp_string(""); // Keywords

        w.into_bytes()
    }
}
