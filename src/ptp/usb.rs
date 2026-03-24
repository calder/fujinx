use std::time::Duration;

use crate::{Error, Result};

const FUJI_VENDOR_ID: u16 = 0x04CB;
const PTP_INTERFACE_CLASS: u8 = 6;
const USB_TIMEOUT: Duration = Duration::from_secs(10);
const PTP_CONTAINER_COMMAND: u16 = 1;
const PTP_CONTAINER_DATA: u16 = 2;
const PTP_CONTAINER_RESPONSE: u16 = 3;
const PTP_HEADER_LEN: usize = 12;
const PTP_RECV_BUF: usize = 512 * 1024;

pub type UsbDevice = rusb::Device<rusb::GlobalContext>;

/// List all connected Fujifilm USB devices.
pub fn detect() -> Result<Vec<UsbDevice>> {
    let devices = rusb::devices().map_err(|e| Error(e.to_string()))?;
    let mut result = Vec::new();

    for device in devices.iter() {
        let desc = device
            .device_descriptor()
            .map_err(|e| Error(e.to_string()))?;
        if desc.vendor_id() == FUJI_VENDOR_ID {
            result.push(device);
        }
    }

    Ok(result)
}

/// Result of a PTP transaction.
pub(crate) struct TransactionResult {
    pub response_code: u16,
    pub data: Vec<u8>,
}

pub(crate) struct UsbTransport {
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    iface: u8,
    ep_in: u8,
    ep_out: u8,
}

impl UsbTransport {
    /// Open a USB transport to the camera.
    ///
    /// On macOS, the system's `ptpcamerad` daemon automatically claims PTP
    /// devices.  We kill it first so libusb can claim the interface.  The
    /// daemon respawns on its own when the next system-level camera app
    /// needs it.
    pub fn open(device: &UsbDevice) -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("killall").arg("ptpcamerad").output().ok();
            std::thread::sleep(Duration::from_millis(200));
        }

        let handle = device.open().map_err(|e| Error(e.to_string()))?;

        let config = device
            .active_config_descriptor()
            .map_err(|e| Error(e.to_string()))?;

        let mut iface_num = None;
        let mut ep_in = None;
        let mut ep_out = None;

        for iface in config.interfaces() {
            for desc in iface.descriptors() {
                if desc.class_code() == PTP_INTERFACE_CLASS {
                    iface_num = Some(desc.interface_number());
                    for ep in desc.endpoint_descriptors() {
                        match (ep.direction(), ep.transfer_type()) {
                            (rusb::Direction::In, rusb::TransferType::Bulk) => {
                                ep_in = Some(ep.address());
                            }
                            (rusb::Direction::Out, rusb::TransferType::Bulk) => {
                                ep_out = Some(ep.address());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let iface = iface_num.ok_or_else(|| Error("no PTP interface found".into()))?;
        let ep_in = ep_in.ok_or_else(|| Error("no bulk-in endpoint found".into()))?;
        let ep_out = ep_out.ok_or_else(|| Error("no bulk-out endpoint found".into()))?;

        if handle.kernel_driver_active(iface).unwrap_or(false) {
            handle
                .detach_kernel_driver(iface)
                .map_err(|e| Error(e.to_string()))?;
        }

        handle
            .claim_interface(iface)
            .map_err(|e| Error(e.to_string()))?;

        Ok(Self {
            handle,
            iface,
            ep_in,
            ep_out,
        })
    }

    pub fn execute(
        &mut self,
        operation: u16,
        params: &[u32],
        transaction_id: u32,
        data_out: Option<&[u8]>,
    ) -> Result<TransactionResult> {
        let param_bytes: Vec<u8> = params.iter().flat_map(|p| p.to_le_bytes()).collect();
        self.send_raw(&build_container(
            PTP_CONTAINER_COMMAND,
            operation,
            transaction_id,
            &param_bytes,
        ))?;

        if let Some(payload) = data_out {
            self.send_raw(&build_container(
                PTP_CONTAINER_DATA,
                operation,
                transaction_id,
                payload,
            ))?;
        }

        let buf = self.recv_container()?;
        let container_type = u16::from_le_bytes([buf[4], buf[5]]);

        if container_type == PTP_CONTAINER_DATA {
            let container_len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
            let data = buf[PTP_HEADER_LEN..container_len].to_vec();

            let resp_code = if buf.len() > container_len {
                parse_response_code(&buf[container_len..])?
            } else {
                parse_response_code(&self.recv_container()?)?
            };

            Ok(TransactionResult {
                response_code: resp_code,
                data,
            })
        } else if container_type == PTP_CONTAINER_RESPONSE {
            Ok(TransactionResult {
                response_code: parse_response_code(&buf)?,
                data: Vec::new(),
            })
        } else {
            Err(Error(format!(
                "unexpected container type {}",
                container_type
            )))
        }
    }

    /// Reset the underlying USB connection, clearing any stale device state.
    pub fn reset(&mut self) -> Result<()> {
        self.handle.reset().map_err(|e| Error(e.to_string()))
    }

    fn send_raw(&self, data: &[u8]) -> Result<()> {
        let mut offset = 0;
        while offset < data.len() {
            let n = self
                .handle
                .write_bulk(self.ep_out, &data[offset..], USB_TIMEOUT)
                .map_err(|e| Error(e.to_string()))?;
            offset += n;
        }

        Ok(())
    }

    /// Read a complete PTP container, which may span multiple USB
    /// bulk transfers.
    fn recv_container(&self) -> Result<Vec<u8>> {
        let mut chunk = vec![0u8; PTP_RECV_BUF];
        let n = self
            .handle
            .read_bulk(self.ep_in, &mut chunk, USB_TIMEOUT)
            .map_err(|e| Error(e.to_string()))?;
        if n < PTP_HEADER_LEN {
            return Err(Error("response too short".into()));
        }

        let container_len = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as usize;

        let mut buf = Vec::with_capacity(container_len);
        buf.extend_from_slice(&chunk[..n]);

        while buf.len() < container_len {
            let n = self
                .handle
                .read_bulk(self.ep_in, &mut chunk, USB_TIMEOUT)
                .map_err(|e| Error(e.to_string()))?;
            buf.extend_from_slice(&chunk[..n]);
        }

        Ok(buf)
    }
}

impl Drop for UsbTransport {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(self.iface);
    }
}

fn build_container(container_type: u16, code: u16, transaction_id: u32, payload: &[u8]) -> Vec<u8> {
    let len = (PTP_HEADER_LEN + payload.len()) as u32;
    let mut buf = Vec::with_capacity(len as usize);
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(&container_type.to_le_bytes());
    buf.extend_from_slice(&code.to_le_bytes());
    buf.extend_from_slice(&transaction_id.to_le_bytes());
    buf.extend_from_slice(payload);

    buf
}

fn parse_response_code(data: &[u8]) -> Result<u16> {
    if data.len() < PTP_HEADER_LEN {
        return Err(Error("response too short".into()));
    }
    let container_type = u16::from_le_bytes([data[4], data[5]]);
    if container_type != PTP_CONTAINER_RESPONSE {
        return Err(Error(format!(
            "expected response container (type 3), got type {}",
            container_type
        )));
    }

    Ok(u16::from_le_bytes([data[6], data[7]]))
}
