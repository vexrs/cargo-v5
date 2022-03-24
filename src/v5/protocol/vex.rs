use std::io::{Read, Write};
use anyhow::{Result, anyhow};
use std::time::{Duration, SystemTime};
use crc::Algorithm;

/// Vex uses CRC16/XMODEM as the CRC16.
const VEX_CRC16: Algorithm<u16> = Algorithm {
    poly: 0x1021,
    init: 0x0000,
    refin: false,
    refout: false,
    xorout: 0x0000,
    check: 0x31c3,
    residue: 0x0000,
};

/// Represents the type of a vex device
pub enum VexDeviceType {
    User,
    System,
    Joystick,
    Unknown
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq)]
#[repr(u8)]
pub enum VexACKType {
    ACK = 0x76,
    NACK_CRC_ERROR = 0xCE,
    NACK_PAYLOAD_SHORT = 0xD0,
    NACK_TRANSFER_SIZE_TOO_LARGE = 0xD1,
    NACK_PROGRAM_CRC_FAILED = 0xD2,
    NACK_PROGRAM_FILE_ERROR = 0xD3,
    NACK_UNINITIALIZED_TRANSFER = 0xD4,
    NACK_INITIALIZATION_INVALID = 0xD5,
    NACK_LENGTH_MOD_FOUR_NZERO = 0xD6,
    NACK_ADDR_NO_MATCH = 0xD7,
    NACK_DOWNLOAD_LENGTH_NO_MATCH = 0xD8,
    NACK_DIRECTORY_NO_EXIST = 0xD9,
    NACK_NO_FILE_ROOM = 0xDA,
    NACK_FILE_ALREADY_EXISTS = 0xDB,
}


/// Represents a vex device command
#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum VexDeviceCommand {
    ExecuteFile = 0x18,
    Extended = 0x56,
}



impl PartialEq<u8> for VexDeviceCommand {
    fn eq(&self, other: &u8) -> bool {
        *self as u8 == *other
    }
}
impl PartialEq<VexDeviceCommand> for VexDeviceCommand {
    fn eq(&self, other: &VexDeviceCommand) -> bool {
        *self as u8 == *other as u8
    }
}
impl PartialEq<VexDeviceCommand> for u8 {
    fn eq(&self, other: &VexDeviceCommand) -> bool {
        *self == *other as u8
    }
}


/// Wraps any struct that implements both read and write
/// traits. Allows sending vex device commands. 
pub struct VexProtocolWrapper<T> 
    where T: Read + Write {
    device_type: VexDeviceType,
    wraps: T
}

impl<T> VexProtocolWrapper<T> 
    where T: Read + Write {

    /// Initializes a new VexProtocolWrapper
    pub fn new(device_type: VexDeviceType, wraps: T) -> VexProtocolWrapper<T> {
        VexProtocolWrapper {
            device_type,
            wraps
        }
    }

    /// Receives an extended packet from the vex device
    pub fn receive_extended(&mut self, timeout: Option<Duration>) -> Result<(VexDeviceCommand, Vec<u8>, Vec<u8>)> {
        
        // Receive simple data
        let data = self.receive_simple(timeout)?;
        
        let crc = crc::Crc::<u16>::new(&VEX_CRC16);

        // Check the crc of the entirety of the recieved data
        let chk = crc.checksum(&data.2);
        if chk != 0 {
            return Err(anyhow!("CRC failed on response."))
        }

        // Verify that this is an extended command
        if data.0 != VexDeviceCommand::Extended {
            return Err(anyhow!("Unexpected packet recieved from device. Expected Extended, got other"));
        }

        // Get the ack result
        let ack = data.1[0];

        // Try to turn the ack into a member of the enum
        let ack: VexACKType = match num::FromPrimitive::from_u8(ack) {
            Some(a) => a,
            None => {
                return Err(anyhow!("Device did not ack with unknown response {}", ack));
            }
        };

        // If it is not an ACK, and we are not exited by now
        // than it is a NACK
        if ack != VexACKType::ACK {
            return Err(anyhow!("Device NACKED with response {:02x}", ack as u8));
        }
        
        // Strip out the payload
        let payload = Vec::from(&data.1[1..]);

        Ok((VexDeviceCommand::Extended, payload, data.2))
    }

    /// Sends an extended packet to the vex device
    pub fn send_extended(&mut self, command: VexDeviceCommand, data: Vec<u8>) -> Result<usize> {
        
        // Create the payload
        let payload = self.create_extended_packet(command, data)?;
        
        // Send the payload and return the length of the data sent
        self.send_simple(VexDeviceCommand::Extended, payload)
    }

    /// Receives a simple packet from the vex device
    pub fn receive_simple(&mut self, timeout: Option<Duration>) -> Result<(VexDeviceCommand, Vec<u8>, Vec<u8>)> {

        // Use default timeout if none was provided
        let timeout = match timeout {
            Some(t) => t,
            None => Duration::new(0,100000000)
        };

        

        // This is the expected header in the response:
        let expected_header: [u8;2] = [0xAA,0x55];
        let mut header_index = 0;

        // Set the duration for the next timeout
        let then = SystemTime::now() + timeout;

        

        // Iterate over recieving single bytes untill we recieve the header
        while header_index < 2 {
            // Recieve a single byte
            let mut b: [u8; 1] = [0];
            self.wraps.read_exact(&mut b)?;

            // If the byte is equivilent to the current index in expected header
            // then increment the current index. if not, then set it back to zero
            if b[0] == expected_header[header_index] {
                header_index += 1;
            } else {
                header_index = 0;
            }

            // If the timeout is elapsed then return an error
            if !then.elapsed().unwrap_or(Duration::new(0, 0)).is_zero() && header_index < 3 {
                return Err(anyhow!("Unable to find response header in timeout, so unable to recieve data from device."));
            }
        }

        // Create a variable containing the entire response recieved
        let mut rx = Vec::from(expected_header);

        
        // Read in the next two bytes
        let mut buf: [u8; 2] = [0, 0];
        self.wraps.read_exact(&mut buf)?;

        // Extract the command and the length of the packet
        let command = buf[0];
        let mut length: u16 = buf[1].into();

        rx.extend(buf);
        
        // If this is an extended command
        if command == VexDeviceCommand::Extended {
            // Then extract the lower byte of the length
            let mut b: [u8; 1] = [0];
            self.wraps.read_exact(&mut b)?;
            rx.extend(b);

            let b: u16 = b[0].into();

            // And append it to the length
            length <<= 8;
            length |= b;
        }

        // Read in the rest of the payload
        let mut payload: Vec<u8> = vec![0; length.into()];
        self.wraps.read(&mut payload)?; // We do not care about size here. Some commands may send less data than needed.
        rx.extend(payload.clone());

        // Try to convert the command into it's enum format
        let command: VexDeviceCommand =  match num::FromPrimitive::from_u8(command) {
            Some(c) => c,
            None => {
                return Err(anyhow!("Unknown command {}", command));
            }
        };
        
        // Return the command and the payload
        Ok((command, payload, rx))
    }

    /// Sends a simple packet to the vex device
    pub fn send_simple(&mut self, command: VexDeviceCommand, data: Vec<u8>) -> Result<usize>{

        // Create the packet
        let mut packet = self.create_packet(command);

        // Add the data to the packet
        packet.append(&mut data.clone());
        
        
        // Write the data
        self.wraps.write_all(&mut packet)?;
        

        // Flush all pending writes on the buffer.
        self.wraps.flush()?;
        
        
        // Return the length of the data sent
        Ok(packet.len())
    }

    /// Creates a simple packet with a magic number
    /// and a command message for a vex device.
    fn create_packet(&self, msg: VexDeviceCommand) -> Vec<u8> {
        // Create a vec of bytes containing the magic number (or at least we assume it is)
        // and the command code.
        vec![0xc9, 0x36, 0xb8, 0x47, msg as u8]
    }

    /// Creates an extended packet
    fn create_extended_packet(&self, command: VexDeviceCommand, payload: Vec<u8>) -> Result<Vec<u8>> {

        let mut packet = Vec::<u8>::new();

        packet.push(command as u8);

        // Cache this value because it will not change.
        let payload_length: u16 = payload.len().try_into()?;

        // If we are larger than an 8-bit signed integer
        // then split the length into two halves
        if payload_length > 0x80 {
            packet.extend(payload_length.to_le_bytes())
        } else {
            packet.push(payload_length as u8);
        }

        // Add the payload
        let mut pc = payload.clone();
        packet.append(&mut pc);

        // Generate the payload as it would appear when sent over the wire
        let mut payload_proper = Vec::<u8>::new();
        payload_proper.extend(self.create_packet(VexDeviceCommand::Extended));
        payload_proper.extend(packet.clone());

        // Compute the CRC16
        let crc = crc::Crc::<u16>::new(&VEX_CRC16);
        let mut digest = crc.digest();
        digest.update(&payload_proper);
        let calc = digest.finalize();

        // Pack the crc into the packet
        packet.push(calc.checked_shr(8).unwrap_or(0) as u8);
        packet.push((calc & 0xff) as u8);
        

        Ok(packet)
    }
}