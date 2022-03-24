use crate::v5::protocol::{
    VexProtocolWrapper,
    VexDeviceCommand
};

use crate::v5::device::{
    V5DeviceVersion, VexProduct,
    V5BrainFlags, V5ControllerFlags
};

use std::io::{Read, Write};

use anyhow::{Result,anyhow};

/// This struct wraps a vex protocol wrapper
/// to provide high-level access to the VEX device.
#[derive(Clone, Debug)]
pub struct VexV5Device<T: Write + Read> {
    wraps: VexProtocolWrapper<T>
}

/// This trait contains functions that all vex v5 devices have
/// in common.
impl<T: Write + Read> VexV5Device<T> {

    /// Initializes a new device
    pub fn new(wraps: VexProtocolWrapper<T>) -> VexV5Device<T> {
        VexV5Device {
            wraps
        }
    }

    /// Gets the version of the device
    pub fn get_device_version(&mut self) -> Result<V5DeviceVersion> {
        

        // Request system information
        self.wraps.send_simple(VexDeviceCommand::GetSystemVersion, Vec::new())?;
        let data = self.wraps.receive_simple(None)?;
        
        // Seperate out the version data
        let vs = data.1;

        // Parse the version data
        let ver = V5DeviceVersion {
            system_version: (vs[0], vs[1], vs[2], vs[3], vs[4]),
            product_type: VexProduct::try_from((vs[5], vs[6]))?,
        };
        

        Ok(ver)
    }



    /// Checks if we are connected to the brain wirelessly.
    pub fn is_wireless(&mut self) -> Result<bool> {
        // Get device version info
        let info = self.get_device_version()?;

        // If it is a controller and connected wirelessly then return true
        // if not, we are not using wireless
        match info.product_type {
            VexProduct::V5Controller(f) => Ok(f.contains(V5ControllerFlags::CONNECTED_WIRELESS)),
            _ => Ok(false)
        }
    }
}
