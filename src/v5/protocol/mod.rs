pub mod vex;
pub use vex::VexProtocolWrapper;


use crc::Algorithm;

/// Vex uses CRC16/XMODEM as the CRC16.
pub const VEX_CRC16: Algorithm<u16> = crc::CRC_16_XMODEM;

/// Vex uses a parametric CRC32 that I found on page
/// 6 of this document: 
/// https://www.matec-conferences.org/articles/matecconf/pdf/2016/11/matecconf_tomsk2016_04001.pdf
pub const VEX_CRC32: Algorithm<u32> = Algorithm {
    poly: 0x04C11DB7,
    init: 0x00000000,
    refin: false,
    refout: false,
    xorout: 0x00000000,
    check: 0x89A1897F,
    residue: 0x00000000,
};

/// Represents the type of a vex device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    NACKCrcError = 0xCE,
    NACKPayloadShort = 0xD0,
    NACKTransferSizeTooLarge = 0xD1,
    NACKProgramCrcFailed = 0xD2,
    NACKProgramFileError = 0xD3,
    NACKUninitializedTransfer = 0xD4,
    NACKInitializationInvalid = 0xD5,
    NACKLengthModFourNzero = 0xD6,
    NACKAddrNoMatch = 0xD7,
    NACKDownloadLengthNoMatch = 0xD8,
    NACKDirectoryNoExist = 0xD9,
    NACKNoFileRoom = 0xDA,
    NACKFileAlreadyExists = 0xDB,
}

/// Represents a vex device command
#[repr(u8)]
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum VexDeviceCommand {
    OpenFile = 0x11,
    ExitFile = 0x12,
    WriteFile = 0x13,
    ReadFile = 0x14,
    SetLinkedFilename = 0x15,
    ExecuteFile = 0x18,
    GetMetadataByFilename = 0x19,
    Extended = 0x56,
    GetSystemVersion = 0xA4,
}


/// Represents a flag that tells the brain what to do
/// after a file transfer is complete
pub enum VexFiletransferFinished {
    DoNothing = 0b0,
    RunProgram = 0b1,
    ShowRunScreen = 0b11,
}

impl Default for VexFiletransferFinished {
    fn default() -> Self {
        VexFiletransferFinished::DoNothing
    }
}

