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


    /// Reads n bytes from the file
    pub fn read_len(&self, offset: u32, n_bytes: u16) -> Result<Vec<u8>> {

        // Pad out the number of bytes to be a multiple of four
        let n_bytes_pad = (n_bytes + 3) & !3;

        // Create a payload containing the offset
        // and the number of bytes to read
        let payload = bincode::serialize(&(offset, n_bytes_pad))?;

        // Send the read command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ReadFile, payload)?;

        // Recieve the response
        let response = self.device.borrow_mut().receive_extended(self.timeout, ResponseCheckFlags::CRC)?;

        // Truncate to requested data (Ignore the integer sent in the first four bytes)
        let offset = 4;
        let data = response.1[offset..offset + n_bytes as usize].to_vec();

        Ok(data)
    }

    /// Reads the entire file
    pub fn read_all(&self) -> Result<Vec<u8>> {
        // Create the buffer to store data in
        let mut data = Vec::<u8>::new();

        let max_size: u16 = 512;
        let length = self.transfer_metadata.file_size;

        // Iterate over the file's size in steps of max_packet_size
        for i in (0..length).step_by(max_size.into()) {
            
            // Find the packet size that we want to read in
            let packet_size = if i + <u32>::from(max_size) > length {
                <u16>::try_from(length - i)?
            } else {
                max_size
            };
            
            // Read the data and append it to the buffer
            data.extend(self.read_len(i+self.metadata.addr, packet_size)?);
        }
        Ok(data)
    }

    /// Writes a vector of data to the file
    /// at the specified offset.
    pub fn write_vec(&self, offset: u32, data: Vec<u8>) -> Result<()> {

        // Pad the payload to have a length that is a multiple of four
        let data_pad = (data.len() + 3) & !3;
        let mut data = data.clone();
        data.resize(data_pad, 0);
        let data = data;

        // Create the payload
        let mut payload = bincode::serialize(&offset)?;
        for b in data {
            payload.push(b);
        }

        // Send the write command
        self.device.borrow_mut().send_extended(VexDeviceCommand::WriteFile, payload)?;

        // Recieve and discard the response
        let _response = self.device.borrow_mut().receive_extended(self.timeout, ResponseCheckFlags::CRC)?;
        
        Ok(())
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

        /// Convert the name to ascii
        let file_name = file_name.as_ascii_str()?;
        let mut file_name_bytes: [u8; 24] = [0; 24];
        for (i, b) in file_name.as_slice().iter().enumerate() {
            if (i + 1) > 23 {
                break;
            }
            file_name_bytes[i] = *b as u8;
        }
        file_name_bytes[23] = 0;

        // Resolve the file metadata to it's default value
        let file_metadata = file_metadata.unwrap_or(VexInitialFileMetadata::default());

        // Get a tuple from the file function
        let ft: (u8, u8, u8) = match file_metadata.function {
            VexFileMode::Upload(t, o) => {
                (1, match t {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, o as u8)
            },
            VexFileMode::Download(t, o) => {
                (2, match t {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, o as u8)
            }
        };

        // Pack the payload together
        let payload: (
            u8, u8, u8, u8,
            u32, u32, u32,
            [u8; 4],
            u32, u32,
            [u8; 24],
        ) = (
            ft.0,
            ft.1,
            file_metadata.vid as u8,
            ft.2 | file_metadata.options,
            file_metadata.length,
            file_metadata.addr,
            file_metadata.crc,
            file_metadata.r#type,
            file_metadata.timestamp,
            file_metadata.version,
            file_name_bytes,
        );
        
        let payload = bincode::serialize(&payload)?;

        // Send the request
        self.wraps.borrow_mut().send_extended(VexDeviceCommand::OpenFile, payload)?;

        // Receive the response
        let response = self.wraps.borrow_mut().receive_extended(self.timeout, ResponseCheckFlags::ALL)?;

        // Parse the response
        let response: (u16, u32, u32) = bincode::deserialize(&response.1)?;
        let response = VexFiletransferMetadata {
            max_packet_size: response.0,
            file_size: response.1,
            crc: response.2,
        };

        // If this is opening for write, then 
        // set the linked filename
        if let VexFileMode::Upload(_, _) = file_metadata.function {
            // Create the payload
            let payload: (u8, u8, [u8; 24]) = (
                file_metadata.vid as u8,
                file_metadata.options | ft.2,
                file_name_bytes
            );
            let payload = bincode::serialize(&payload)?;

            // Send the command
            self.wraps.borrow_mut().send_extended(VexDeviceCommand::SetLinkedFilename, payload)?;

        }
        
        // Create the file handle
        let handle = V5FileHandle {
            device: Rc::clone(&self.wraps),
            transfer_metadata: response,
            metadata: file_metadata,
            file_name: file_name.to_ascii_string(),
            position: 0,
            wraps: self,
            timeout: self.timeout,
        };

        // Return the handle
        Ok(handle)
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
