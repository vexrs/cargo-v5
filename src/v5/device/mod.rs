mod device;
use chrono::TimeZone;
pub use device::VexV5Device;
use bitflags::bitflags;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

/// The target to open the file on
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VexFileTarget {
    DDR = 0,
    FLASH = 1,
    SCREEN = 2,
}

/// The mode to open a file on the V5 device with
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VexFileMode {
    /// Open the file for uploading
    Upload(VexFileTarget, bool),
    /// Open the file for downloading
    Download(VexFileTarget, bool),
}


/// Different possible vex VIDs
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum VexVID { // I also have no idea what this is.
    USER = 1,
    SYSTEM = 15,
    RMS = 16, // I believe that robotmesh studio uses this
    PROS = 24, // PROS uses this one
    MW = 32, // IDK what this one is.
}


/// Represents vex file metadata when initiating
/// a transfer
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VexInitialFileMetadata {
    pub function: VexFileMode,
    pub vid: VexVID,
    pub options: u8,
    pub length: u32,
    pub addr: u32,
    pub crc: u32,
    pub r#type: [u8; 4],
    pub timestamp: u32,
    pub version: u32,
}

impl Default for VexInitialFileMetadata {
    fn default() -> Self {
        VexInitialFileMetadata {
            function: VexFileMode::Upload(VexFileTarget::FLASH, true),
            vid: VexVID::USER,
            options: 0,
            length: 0,
            addr: 0x3800000,
            crc: 0,
            r#type: *b"bin\0",
            // Default timestamp to number of seconds after Jan 1 2000
            timestamp: (chrono::Utc::now().timestamp() - chrono::Utc.ymd(2000, 1, 1)
                            .and_hms(0, 0, 0).timestamp()).try_into().unwrap(),
            version: 0,
        }
    }
}

/// File metadata returned from the V5 device
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VexFileMetadata {
    pub idx: u8,
    pub size: u32,
    pub addr: u32,
    pub crc: u32,
    pub r#type: [u8; 4],
    pub timestamp: u32,
    pub version: u32,
    pub filename: [u8; 24],
}

impl Default for VexFileMetadata {
    fn default() -> Self {
        VexFileMetadata {
            idx: 0,
            size: 0,
            addr: 0,
            crc: 0,
            r#type: *b"\0\0\0\0",
            timestamp: 0,
            version: 0,
            filename: [0; 24],
        }
    }
}

/// Metadata for a file transfer
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VexFiletransferMetadata {
    pub max_packet_size: u16,
    pub file_size: u32,
    pub crc: u32,
}

bitflags!{
    /// Configuration flags for the v5 brain
    pub struct V5BrainFlags: u8 {
        const NONE = 0x0;
    }
    /// Configuration flags for the v5 controller
    pub struct V5ControllerFlags: u8 {
        const NONE = 0x0;
        const CONNECTED_CABLE = 0x01; // From testing, this appears to be how it works.
        const CONNECTED_WIRELESS = 0x02;
    }
}

/// This enum is a convenient representation
/// of which type of product the VEX device is.
#[derive(Debug, Clone, Copy)]
pub enum VexProduct {
    V5Brain(V5BrainFlags),
    V5Controller(V5ControllerFlags),
}

impl Into<u8> for VexProduct {
    fn into(self) -> u8 {
        match self {
            VexProduct::V5Brain(_) => 0x10,
            VexProduct::V5Controller(_) => 0x11,
        }
    }
}

impl TryFrom<(u8, u8)> for VexProduct {
    type Error = anyhow::Error;

    fn try_from(value: (u8,u8)) -> Result<VexProduct> {
        match value.0 {
            0x10 => Ok(VexProduct::V5Brain(V5BrainFlags::from_bits(value.1).unwrap_or(V5BrainFlags::NONE))),
            0x11 => Ok(VexProduct::V5Controller(V5ControllerFlags::from_bits(value.1).unwrap_or(V5ControllerFlags::NONE))),
            _ => Err(anyhow!("Invalid product type")),
        }
    }
}


/// This struct represents the version of a vex v5 device
#[derive(Debug, Clone, Copy)]
pub struct V5DeviceVersion {
    pub system_version: (u8, u8, u8, u8, u8),
    pub product_type: VexProduct,
}


/// Enum that represents the channel
/// for the V5 Controller
pub enum V5ControllerChannel {
    /// Used when wirelessly controlling the 
    /// V5 Brain
    PIT,
    /// Used when wirelessly uploading data to the V5
    /// Brain
    UPLOAD,
    /// Used when wirelessly downloading data from the V5
    /// Brain
    DOWNLOAD,
}