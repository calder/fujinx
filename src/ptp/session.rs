use anyhow::{Result, bail};

use super::camera_info::CameraInfo;
use super::dataset::{DatasetReader, DatasetWriter};
use super::usb::{TransactionResult, UsbDevice, UsbTransport};

const PTP_OC_GET_DEVICE_INFO: u16 = 0x1001;
const PTP_OC_OPEN_SESSION: u16 = 0x1002;
const PTP_OC_CLOSE_SESSION: u16 = 0x1003;
const PTP_OC_GET_OBJECT_HANDLES: u16 = 0x1007;
const PTP_OC_GET_OBJECT: u16 = 0x1009;
const PTP_OC_DELETE_OBJECT: u16 = 0x100B;
const PTP_OC_GET_DEVICE_PROP_VALUE: u16 = 0x1015;
const PTP_OC_SET_DEVICE_PROP_VALUE: u16 = 0x1016;

const PTP_RC_OK: u16 = 0x2001;
const PTP_RC_SESSION_ALREADY_OPEN: u16 = 0x201E;

/// An active PTP session.
pub(crate) struct Session {
    transport: UsbTransport,
    transaction_id: u32,
}

impl Session {
    pub fn open(camera: &UsbDevice) -> Result<Self> {
        let mut transport = UsbTransport::open(camera)?;
        let result = transport.execute(PTP_OC_OPEN_SESSION, &[1], 1, None)?;
        if result.response_code == PTP_RC_SESSION_ALREADY_OPEN {
            // Stale session from a previous crash — reset and retry.
            transport.reset()?;
            let result = transport.execute(PTP_OC_OPEN_SESSION, &[1], 1, None)?;
            check_response(PTP_OC_OPEN_SESSION, result.response_code)?;
        } else {
            check_response(PTP_OC_OPEN_SESSION, result.response_code)?;
        }

        Ok(Self {
            transport,
            transaction_id: 1,
        })
    }

    pub fn get_camera_info(&mut self) -> Result<CameraInfo> {
        let result = self.execute(PTP_OC_GET_DEVICE_INFO, &[], None)?;

        CameraInfo::parse(&result.data)
    }

    pub fn get_device_prop_value_raw(&mut self, prop_code: u16) -> Result<Vec<u8>> {
        let result = self.execute(PTP_OC_GET_DEVICE_PROP_VALUE, &[prop_code as u32], None)?;

        Ok(result.data)
    }

    pub fn get_device_prop_value_i16(&mut self, prop_code: u16) -> Result<i16> {
        Ok(self.get_device_prop_value_u16(prop_code)? as i16)
    }

    pub fn get_device_prop_value_u16(&mut self, prop_code: u16) -> Result<u16> {
        let data = self.get_device_prop_value_raw(prop_code)?;
        if data.len() < 2 {
            bail!(
                "expected 2 bytes for property 0x{:04X}, got {}",
                prop_code,
                data.len()
            );
        }

        Ok(u16::from_le_bytes([data[0], data[1]]))
    }

    pub fn get_device_prop_value_string(&mut self, prop_code: u16) -> Result<String> {
        let data = self.get_device_prop_value_raw(prop_code)?;
        let mut r = DatasetReader::new(&data);

        r.read_ptp_string()
    }

    /// Execute a vendor operation with parameters and optional data.
    pub fn vendor_execute(
        &mut self,
        operation: u16,
        params: &[u32],
        data_out: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let result = self.execute(operation, params, data_out)?;

        Ok(result.data)
    }

    pub fn set_device_prop_value_i16(&mut self, prop_code: u16, value: i16) -> Result<()> {
        self.set_device_prop_value_u16(prop_code, value as u16)
    }

    pub fn set_device_prop_value_u16(&mut self, prop_code: u16, value: u16) -> Result<()> {
        self.execute(
            PTP_OC_SET_DEVICE_PROP_VALUE,
            &[prop_code as u32],
            Some(&value.to_le_bytes()),
        )?;

        Ok(())
    }

    pub fn set_device_prop_value_string(&mut self, prop_code: u16, value: &str) -> Result<()> {
        let mut w = DatasetWriter::new();
        w.write_ptp_string(value);
        self.set_device_prop_value_raw(prop_code, &w.into_bytes())
    }

    pub fn set_device_prop_value_raw(&mut self, prop_code: u16, data: &[u8]) -> Result<()> {
        self.execute(
            PTP_OC_SET_DEVICE_PROP_VALUE,
            &[prop_code as u32],
            Some(data),
        )?;

        Ok(())
    }

    pub fn delete_object(&mut self, handle: u32) -> Result<()> {
        self.execute(PTP_OC_DELETE_OBJECT, &[handle], None)?;

        Ok(())
    }

    pub fn get_object_handles(
        &mut self,
        storage_id: u32,
        format_code: u32,
        parent_handle: u32,
    ) -> Result<Vec<u32>> {
        let result = self.execute(
            PTP_OC_GET_OBJECT_HANDLES,
            &[storage_id, format_code, parent_handle],
            None,
        )?;

        parse_u32_array(&result.data)
    }

    pub fn get_object(&mut self, handle: u32) -> Result<Vec<u8>> {
        let result = self.execute(PTP_OC_GET_OBJECT, &[handle], None)?;

        Ok(result.data)
    }

    fn execute(
        &mut self,
        operation: u16,
        params: &[u32],
        data_out: Option<&[u8]>,
    ) -> Result<TransactionResult> {
        let tid = self.next_transaction_id();
        let result = self.transport.execute(operation, params, tid, data_out)?;
        check_response(operation, result.response_code)?;

        Ok(result)
    }

    fn next_transaction_id(&mut self) -> u32 {
        let id = self.transaction_id.wrapping_add(1);
        self.transaction_id = id;

        id
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let tid = self.next_transaction_id();
        let _ = self.transport.execute(PTP_OC_CLOSE_SESSION, &[], tid, None);
    }
}

fn parse_u32_array(data: &[u8]) -> Result<Vec<u32>> {
    let mut r = DatasetReader::new(data);
    let count = r.read_u32()? as usize;
    let mut handles = Vec::with_capacity(count);
    for _ in 0..count {
        handles.push(r.read_u32()?);
    }

    Ok(handles)
}

fn check_response(operation: u16, response_code: u16) -> Result<()> {
    if response_code != PTP_RC_OK {
        bail!("operation 0x{operation:04x} failed: 0x{response_code:04x}");
    }

    Ok(())
}
