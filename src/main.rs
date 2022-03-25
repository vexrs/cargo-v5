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
        .timeout(std::time::Duration::new(5,0))// We handle our own timeouts so a long timeout on the serial side is required.
        .stop_bits(serialport::StopBits::One).open()?;

    let wrapper = VexProtocolWrapper::new(VexDeviceType::System, port);
    let mut device = VexV5Device::new(wrapper);
    let ver = device.get_device_version()?;
    println!("{:?}", ver);

    let file_name = "test.txt";

    

    let data = Vec::<u8>::from(*b"Hello, Culpeper Team 7122A!");
    // Grab a crc32 of the data
    let crc32 = crc::Crc::<u32>::new(&VEX_CRC32).checksum(&data);
    println!("test crc32: {:x}", crc32);

    device.switch_channel(Some(V5ControllerChannel::DOWNLOAD))?;
    
    // Open a test file
    let mut file = device.open(file_name.to_string(), Some(VexInitialFileMetadata {
        function: VexFileMode::Download(VexFileTarget::FLASH, true),
        vid: VexVID::USER,
        options: 0,
        length: data.len() as u32,
        addr: 0x3800000,
        crc: crc32,
        r#type: *b"bin\0",
        timestamp: (chrono::Utc::now().timestamp() - chrono::Utc.ymd(2000, 1, 1)
        .and_hms(0, 0, 0).timestamp()).try_into().unwrap(),
        version: 0x01000000
    }))?;

    

    
    let buf = file.read_all_vec()?;
    println!("{:?}", buf);

    // Convert buf to string
    let s = ascii::AsciiStr::from_ascii(&buf)?.to_string();
    println!("{}",s); 
    

    //file.write_position(0x3800000, data)?;
    
    file.close()?;

    device.switch_channel(Some(V5ControllerChannel::PIT))?;

    // Get the metadata
    let metadata = device.get_file_metadata(file_name.to_string(), None, None)?;
    println!("{:?}", metadata);


    drop(device);
    Ok(())
}