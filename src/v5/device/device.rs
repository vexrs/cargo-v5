use crate::v5::protocol::vex::ResponseCheckFlags;
use crate::v5::protocol::{
    VexProtocolWrapper,
    VexDeviceCommand, VEX_CRC32
};
use crate::v5::device::{
    V5DeviceVersion, VexProduct,
    V5ControllerFlags, VexFileMode,
    VexFileTarget, VexInitialFileMetadata,
    VexFiletransferMetadata
};
use std::io::{Read, Write};
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::{Result, anyhow};
use ascii::{AsciiString, AsAsciiStr};
use super::{VexVID, VexFileMetadata, V5ControllerChannel};



/// This represents a file handle
/// for files on the V5 device.
#[derive(Clone, Debug)]
pub struct V5FileHandle<'a, T> 
    where T: Read + Write {
    device: Rc<RefCell<VexProtocolWrapper<T>>>,
    pub transfer_metadata: VexFiletransferMetadata,
    pub metadata: VexInitialFileMetadata,
    pub file_name: AsciiString,
    position: usize,
    wraps: &'a VexV5Device<T>,
    timeout: Option<std::time::Duration>,
}

impl<'a, T: Write + Read> V5FileHandle<'a, T> {
    /// Closes the file transfer
    pub fn close(&mut self) -> Result<Vec<u8>> {


        // Send the exit command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ExitFile, Vec::<u8>::from([0b11u8]))?;

        // Get the response
        let response = self.device.borrow_mut().receive_extended(self.timeout, ResponseCheckFlags::ALL)?;
        
        // Return the response data
        Ok(response.1)
    }

}




/// This struct wraps a vex protocol wrapper
/// to provide high-level access to the VEX device.
#[derive(Clone, Debug)]
pub struct VexV5Device<T: Write + Read> {
    wraps: Rc<RefCell<VexProtocolWrapper<T>>>,
    timeout: Option<std::time::Duration>
}

/// This trait contains functions that all vex v5 devices have
/// in common.
impl<T: Write + Read> VexV5Device<T> {

    /// Initializes a new device
    pub fn new(wraps: VexProtocolWrapper<T>) -> VexV5Device<T> {

        let mut dev = VexV5Device {
            wraps: Rc::new(RefCell::new(wraps)),
            timeout: None
        };
    
        // Set our default timeout based on wireless status
        dev.timeout  = if dev.is_wireless().unwrap_or(false) {
            Some(std::time::Duration::new(5,0))
        } else {
            None
        };

        dev
    }
        

    /// Switches the channel if this is a controller.
    pub fn switch_channel(&mut self, channel: Option<V5ControllerChannel>) -> Result<()> {

        // If the channel is none, then switch back to pit
        let channel = channel.unwrap_or(V5ControllerChannel::PIT);

        // Send the command
        self.wraps.borrow_mut().send_extended(VexDeviceCommand::SwitchChannel, Vec::<u8>::from([channel as u8]))?;

        // Recieve and discard the response
        let _response = self.wraps.borrow_mut().receive_extended(self.timeout, ResponseCheckFlags::ALL)?;

        Ok(())
    }

    /// Gets the version of the device
    pub fn get_device_version(&mut self) -> Result<V5DeviceVersion> {
        

        // Request system information
        self.wraps.borrow_mut().send_simple(VexDeviceCommand::GetSystemVersion, Vec::new())?;
        let data = self.wraps.borrow_mut().receive_simple(None)?;
        
        // Seperate out the version data
        let vs = data.1;

        // Parse the version data
        let ver = V5DeviceVersion {
            system_version: (vs[0], vs[1], vs[2], vs[3], vs[4]),
            product_type: VexProduct::try_from((vs[5], vs[6]))?,
        };
        

        Ok(ver)
    }

    /// Get metadata for a file from it's name
    pub fn get_file_metadata(&self, file_name: String, vid: Option<VexVID>, options: Option<u8>) -> Result<VexFileMetadata> {
        
        // Resolve default values
        let vid = vid.unwrap_or(VexVID::USER);
        let options = options.unwrap_or(0);

        // Convert the file name into a static length ascii string of length 24
        let mut file_name_bytes = [0u8; 24];
        let file_name = file_name.as_ascii_str()?;
        for (i, b) in file_name.as_slice().iter().enumerate() {
            if (i + 1) > 23 {
                break;
            }
            file_name_bytes[i] = *b as u8;
        }
        file_name_bytes[23] = 0;

        // Pack the data together
        let data = bincode::serialize(&(vid as u8, options, file_name_bytes)).unwrap();

        // Send the request
        self.wraps.borrow_mut().send_extended(VexDeviceCommand::GetMetadataByFilename, data)?;
        let recv = self.wraps.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;

        // Parse the response
        let recv: VexFileMetadata = bincode::deserialize(&recv.1)?;


        Ok(recv)
    }

    /// Opens a file handle on the v5 device
    pub fn open(&mut self, file_name: String, file_metadata: Option<VexInitialFileMetadata>) -> Result<V5FileHandle<T>> {

        

    }

    /// Checks if the device is a controller connected to the brain wirelessly.
    pub fn is_wireless(&mut self) -> Result<bool> {
        // Get device version info
        let info = self.get_device_version()?;

        // If it is a controller and connected wirelessly then return true
        // if not, we are not using wireless
        match info.product_type {
            VexProduct::V5Controller(f) => Ok(f.contains(V5ControllerFlags::CONNECTED_WIRELESS) ||
                            f.contains(V5ControllerFlags::CONNECTED_CABLE)),
            _ => Ok(false)
        }
    }
}
