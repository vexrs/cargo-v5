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
use anyhow::{Result};
use ascii::{AsAsciiStr};

use super::{VexVID, VexFileMetadata};



/// This represents a file handle
/// for files on the V5 device.
#[derive(Clone, Debug)]
pub struct V5FileHandle<T> 
    where T: Read + Write {
    device: Rc<RefCell<VexProtocolWrapper<T>>>,
    transfer_metadata: VexFiletransferMetadata,
    metadata: VexInitialFileMetadata,
    file_name: AsciiString,
    position: usize
}

impl<T: Write + Read> V5FileHandle<T> {
    pub fn close(&mut self) -> Result<Vec<u8>> {


        // Send the exit command
        self.device.borrow_mut().send_extended(VexDeviceCommand::ExitFile, Vec::<u8>::from([0b11u8]))?;

        // Get the response
        let response = self.device.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;
        
        // Return the response data
        Ok(response.1)
    }
}

impl<T: Write + Read> std::io::Read for V5FileHandle<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        
        // Iterate over the length to read in units of the max packet size
        for i in (0..buf.len()).step_by(self.transfer_metadata.max_packet_size.into()) {

            // Determine the current packet size
            let packet_size = self.transfer_metadata.max_packet_size;

            let packet_size = if i + <usize>::from(packet_size) > buf.len() {
                (buf.len() - i).try_into().unwrap_or(0xFFFF)
            } else {
                packet_size
            };

            // Pad the number of bytes to the nearest multiple of 4
            // This is needed due to the behavior of the v5 requiring this,
            // not because of any special operation.
            let padded_size = (packet_size + 3) & (!0x3u16);

            // Pack the payload for the command together
            let payload: (u32, u16) = (<u32>::try_from(i + self.position).unwrap(), padded_size);
            let payload = bincode::serialize(&payload).unwrap();

            // Send the command
            self.device.borrow_mut().send_extended(VexDeviceCommand::ReadFile, payload).unwrap();

            // Get the response
            let recv = self.device.borrow_mut().receive_extended(None, ResponseCheckFlags::NONE).unwrap();
            
            
            // Unpack the response
            //let recv_len: (u32) = bincode::deserialize(&recv.1[0..4]).unwrap();
            
            // Truncate the data to the requested length
            let recv_data = recv.1[3..3+packet_size as usize].to_vec();
            
            // Copy the data to the buffer
            buf[i..i + <usize>::from(packet_size)].copy_from_slice(&recv_data);


        }
        // Update position
        self.position += buf.len();

        Ok(buf.len())
    }
}

impl<T: Write + Read> std::io::Write for V5FileHandle<T> {
    fn flush(&mut self) -> Result<(), std::io::Error>{
        self.device.borrow_mut().flush().unwrap();
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error>{

        // Create a new mutable vector for the contents of buf
        let mut buf_vec = buf.to_vec();

        // Pad it out to a length that is a multiple of four
        let padded_size = (buf_vec.len() + 3) & (!0x3usize);
        buf_vec.resize(padded_size, 0);

        // Take the CRC32 of the buffer
        let crc32 = crc::Crc::<u32>::new(&VEX_CRC32).checksum(buf);

        // Get the buffer size
        let file_len = buf.len();

        // Setup a linked file name
        

        Ok(0)
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
        let file_name = file_name.as_ascii_str()?;
        for (i, b) in file_name.as_slice().iter().enumerate() {
            if (i + 1) > 23 {
                break;
            }
            file_name_bytes[i] = *b as u8;
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

        // Make the request
        self.wraps.borrow_mut().send_extended(VexDeviceCommand::OpenFile, data)?;
        
        let recv = self.wraps.borrow_mut().receive_extended(None, ResponseCheckFlags::ALL)?;
        
        // Unpack the payload
        let recv: VexFiletransferMetadata = bincode::deserialize(&recv.1)?;
        

        // Create the file handle
        Ok(V5FileHandle {
            device: Rc::clone(&self.wraps),
            transfer_metadata: recv,
            metadata,
            file_name,
            position: 0
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
