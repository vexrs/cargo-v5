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
use super::{VexVID, VexFileMetadata};



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
    wraps: &'a VexV5Device<T>
}

impl<'a, T: Write + Read> V5FileHandle<'a, T> {
    /// Closes the file transfer
    pub fn close(&mut self) -> Result<Vec<u8>> {


        // Send the exit command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ExitFile, Vec::<u8>::from([0b11u8]))?;

        // Get the response
        let response = self.device.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;
        
        // Return the response data
        Ok(response.1)
    }

    /// Flushes the write buffer
    /// or, in this case, the serial buffer
    pub fn flush(&mut self) -> Result<()> {
        self.device.borrow_mut().flush()?;
        Ok(())
    }

    /// Reads in a range of bytes from the file
    /// and returns the data as a Vec<u8>
    pub fn read_range(&mut self, start: u32, n_bytes: u16) -> Result<Vec<u8>> {

        // Pad out the number of bytes to be a multiple of four
        let n_bytes = (n_bytes + 3) & !0x3;


        // Pack together the payload
        let payload: (u32, u16) = (start, n_bytes);
        let payload = bincode::serialize(&payload)?;

        // Send the command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ReadFile, payload)?;

        // Get the response
        let response = self.device.borrow_mut().receive_extended(None, ResponseCheckFlags::NONE)?;

        // Cut out the unneeded data
        let ret = response.1[..n_bytes.into()].to_vec();

        // Return the response data
        Ok(ret)
    }

    /// Writes to the file at a specific position
    pub fn write_position(&mut self, start: u32, buf: Vec<u8>) -> Result<()> {

        // Pad the payload so that the length is a multiple of four
        let mut buf = buf;
        buf.resize((buf.len() + 3) & !0x3, 0x00);

        // Pack together the payload
        let payload: (u32) = (start);
        let mut payload = bincode::serialize(&payload)?;
        for b in buf {
            payload.push(b);
        }
        println!("{:?}", payload);

        // Send the command
        self.device.borrow_mut().send_extended(VexDeviceCommand::WriteFile, payload)?;


        // Recieve response
        let _response = self.device.borrow_mut().receive_extended(Some(std::time::Duration::new(2,0)), ResponseCheckFlags::ALL)?;

        Ok(())
    }

}




/// This struct wraps a vex protocol wrapper
/// to provide high-level access to the VEX device.
#[derive(Clone, Debug)]
pub struct VexV5Device<T: Write + Read> {
    wraps: Rc<RefCell<VexProtocolWrapper<T>>>
}

/// This trait contains functions that all vex v5 devices have
/// in common.
impl<T: Write + Read> VexV5Device<T> {

    /// Initializes a new device
    pub fn new(wraps: VexProtocolWrapper<T>) -> VexV5Device<T> {
        VexV5Device {
            wraps: Rc::new(RefCell::new(wraps))
        }
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

        // Convert the file name to a zero-terminated string 24 bytes long
        let mut file_name_bytes = [0u8; 24];
        let file_name_ascii = file_name.as_ascii_str()?;
        for (i, byte) in file_name_ascii.clone().as_slice().iter().enumerate() {
            if (i + 1) > 23 {
                break;
            }
            file_name_bytes[i] = *byte as u8;
        }
        file_name_bytes[23] = 0;

        // Resolve file metadata to defaults
        let file_metadata = file_metadata.unwrap_or(VexInitialFileMetadata::default());
        let metadata = file_metadata.clone();
        

        // Get a tuple of the function and target
        let ft: (u8, u8, bool) = match file_metadata.function {
            VexFileMode::Upload(target, overwrite) => {
                (1, match target {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, overwrite)
            },
            VexFileMode::Download(target, overwrite) => {
                (2, match target {
                    VexFileTarget::DDR => 0,
                    VexFileTarget::FLASH => 1,
                    VexFileTarget::SCREEN => 2,
                }, overwrite)
            }
        };

        // The payload to pack
        let payload: (u8, u8, u8, u8, u32, u32, u32,
                [u8; 4], u32, u32, [u8; 24]) = (
                    ft.0,
                    ft.1,
                    file_metadata.vid as u8,
                    ft.2 as u8 | file_metadata.options,
                    file_metadata.length,
                    file_metadata.addr,
                    file_metadata.crc,
                    file_metadata.r#type,
                    file_metadata.timestamp,
                    file_metadata.version,
                    file_name_bytes,
                );
        
        // Pack the payload
        let data = bincode::serialize(&payload)?;

        println!("{:?}", data);


        // Make the request
        self.wraps.borrow_mut().send_extended(VexDeviceCommand::OpenFile, data)?;
        
        let recv = self.wraps.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;
        
        // Unpack the payload
        let recv: VexFiletransferMetadata = bincode::deserialize(&recv.1)?;
        
        // If we are opening a file for upload, then setup the linked file name
        if let VexFileMode::Upload(_,_) = file_metadata.function {
            // Pack the data
            let payload: (u8, u8, [u8; 24]) = (
                file_metadata.vid as u8,
                0,
                file_name_bytes
            );
            let payload = bincode::serialize(&payload)?;

            // Send the command
            self.wraps.borrow_mut().send_extended(VexDeviceCommand::SetLinkedFilename, payload)?;

            // Recieve and discard response
            self.wraps.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;
        }

        // Create the file handle
        Ok(V5FileHandle {
            device: Rc::clone(&self.wraps),
            transfer_metadata: recv,
            metadata,
            file_name: file_name_ascii.to_ascii_string(),
            position: 0,
            wraps: self
        })
    }

    /// Checks if the device is a controller connected to the brain wirelessly.
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
