use vex_v5_serial::v5::protocol::{
    VEX_CRC32,
    vex::VexProtocolWrapper,
};
use vex_v5_serial::v5::device::{VexV5Device, VexInitialFileMetadata, VexFileMode, VexFileTarget,
    VexVID, V5ControllerChannel};
use anyhow::Result;

fn main() -> Result<()>{
    let port = serialport::new("/dev/ttyACM0", 115200)
        .parity(serialport::Parity::None)
        .timeout(std::time::Duration::new(10,0))// We handle our own timeouts so a long timeout on the serial side is required.
        .stop_bits(serialport::StopBits::One).open()?;

    let wrapper = VexProtocolWrapper::new(port);
    let mut device = VexV5Device::new(wrapper);
    let ver = device.get_device_version()?;
    println!("{:?}", ver);

    let file_name = "test.txt";

    device.switch_channel(Some(V5ControllerChannel::UPLOAD))?;

    let data = b"I like wireless upload.".to_vec();
    //let data = data[0..512].to_vec();
    // Calculate the crc32
    let crc32 = crc::Crc::<u32>::new(&VEX_CRC32).checksum(&data);

    println!("{:x}", crc32);
    let addr = 0x3800000;

    // Open a file
    let mut file = device.open(
        file_name.to_string(),
        Some(VexInitialFileMetadata {
            function: VexFileMode::Upload(VexFileTarget::FLASH, true),
            vid: VexVID::USER,
            options: 0,
            length: data.len() as u32,
            addr,
            crc: crc32,
            r#type: *b"bin\0",
            timestamp: <u32>::try_from(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs())?,
            version: 0x01000000,
        })
    )?;
    
    file.write_all(data)?;
    
    

    
    // Close the file
    file.close()?;

    device.switch_channel(Some(V5ControllerChannel::PIT))?;

    drop(device);
    Ok(())
}