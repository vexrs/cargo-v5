use std::io::Read;
use chrono::TimeZone;
use vex_v5_serial::v5::protocol::vex::ResponseCheckFlags;
use vex_v5_serial::v5::protocol::{VexFiletransferFinished, VEX_CRC32};
use vex_v5_serial::v5::protocol::{
    VexDeviceCommand,
    VexDeviceType,
    vex::VexProtocolWrapper
};
use vex_v5_serial::v5::device::{VexV5Device, VexInitialFileMetadata, VexFileMode, VexFileTarget,
    VexVID, V5ControllerChannel};
use anyhow::Result;
use ascii::AsAsciiStr;


fn main() -> Result<()>{
    let port = serialport::new("/dev/ttyACM0", 115200)
        .parity(serialport::Parity::None)
        .timeout(std::time::Duration::new(10,0))// We handle our own timeouts so a long timeout on the serial side is required.
        .stop_bits(serialport::StopBits::One).open()?;

    let wrapper = VexProtocolWrapper::new(VexDeviceType::System, port);
    let mut device = VexV5Device::new(wrapper);
    let ver = device.get_device_version()?;
    println!("{:?}", ver);

    let file_name = "test.txt";

    

    let file_contents = b"Hello, World!".to_vec();

    // Calculate the crc32
    let crc32 = crc::Crc::<u32>::new(&VEX_CRC32).checksum(&file_contents);

    println!("{:x}", crc32);

    // Open a file
    let mut file = device.open(
        file_name.to_string(),
        Some(VexInitialFileMetadata {
            function: VexFileMode::Upload(VexFileTarget::FLASH, true),
            vid: VexVID::USER,
            options: 0,
            length: file_contents.len() as u32,
            addr: 0x03800000,
            crc: crc32,
            r#type: *b"bin\0",
            timestamp: <u32>::try_from(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs())?,
            version: 0x01000000,
        })
    )?;
    
    file.write_vec(0x03800000, file_contents)?;
    
    // Close the file
    file.close()?;

    drop(device);
    Ok(())
}