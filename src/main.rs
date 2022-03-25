use std::io::Read;

use vex_v5_serial::v5::protocol::VexFiletransferFinished;
use vex_v5_serial::v5::protocol::{
    VexDeviceCommand,
    VexDeviceType,
    vex::VexProtocolWrapper
};
use vex_v5_serial::v5::device::{VexV5Device, VexFileMetadata, VexFileMode, VexFileTarget,
    VexVID};
use anyhow::Result;



fn main() -> Result<()>{
    let port = serialport::new("/dev/ttyACM0", 115200)
        .parity(serialport::Parity::None)
        .timeout(std::time::Duration::new(0,100000000))
        .stop_bits(serialport::StopBits::One).open()?;

    let wrapper = VexProtocolWrapper::new(VexDeviceType::System, port);
    let mut device = VexV5Device::new(wrapper);
    let ver = device.get_device_version()?;
    println!("{:?}", ver);

    // Open a test file
    let mut file = device.open("test.txt".to_string(), Some(VexFileMetadata {
        function: VexFileMode::Download(VexFileTarget::FLASH, false),
        vid: VexVID::USER,
        options: 0,
        length: 0,
        addr: 0,
        crc: 0,
        r#type: *b"bin\0",
        timestamp: 0,
        version: 0x01000000
    }))?;

    let mut buf = [0u8;16];
    file.read(&mut buf)?;
    println!("{:?}", buf);

    // Convert buf to string
    let s = ascii::AsciiStr::from_ascii(&buf)?.to_string();
    println!("{}",s);

    file.close()?;

    //let to_serialize: (u8, u8, [u8; 24]) = (1, 0, *b"slot_1.bin\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    //let data = bincode::serialize(&to_serialize)?;
    //wrapper.send_extended(VexDeviceCommand::ExecuteFile, data)?;
    //let data = wrapper.receive_extended(Some(std::time::Duration::new(5,0)))?;
    

    drop(device);
    Ok(())
}