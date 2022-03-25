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

    

    let data = Vec::<u8>::from(*b"Hello, Culpeper Team 7122A! This is a really long message that I want to keep really super duper long so that I can test how well my system works. This is because it needs to be longer than 512 bytes so I can test overflow. There are two routes in the code for detecting overflow that I have not tested yet so I will make this 'file' super long. This text is always encoded in ascii for some reason. It could be in UTF-8, but I keep it as ascii for two reasons: Compatibility with other software like RMS and PROS, and so that if you are writign a slot_x.ini the information shows up in the UI correctly. I will copy and pase this a second time.");
    // Grab a crc32 of the data
    let crc32 = crc::Crc::<u32>::new(&VEX_CRC32).checksum(&data);
    println!("test crc32: {:x}", crc32);

    

    // Get the metadata
    let metadata = device.get_file_metadata(file_name.to_string(), None, None)?;
    println!("{:?}", metadata);


    drop(device);
    Ok(())
}