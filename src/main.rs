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

    // Get file metadata
    let metadata = device.get_file_metadata(file_name.to_string(), None, None)?;
    println!("{:?}", metadata);

    // Open a file
    let mut file = device.open(
        file_name.to_string(),
        Some(VexInitialFileMetadata {
            function: VexFileMode::Download(VexFileTarget::FLASH, false),
            vid: VexVID::USER,
            options: 0,
            length: 0,
            addr: 0x3800000,
            crc: 0,
            r#type: *b"txt\0",
            timestamp: 0x0,
            version: 0x0,
        })
    )?;

    let mut buf = Vec::<u8>::new();

    for i in (0..metadata.size).step_by(512) {
        let mut packet_size: u16 = 512;

        if i + <u32>::from(packet_size) > metadata.size {
            packet_size = <u16>::try_from(metadata.size - i)?;
        }

        let data = file.read_len(i+metadata.addr, packet_size)?;
        buf.extend(data);
    }

    println!("{:?}", buf);
    println!("{}", buf.len());

    // Convert buf to ascii string and then print
    let ascii_str = buf.as_ascii_str()?;
    println!("{}", ascii_str);
    println!("{}", ascii_str.len());

    // Close the file
    file.close()?;

    drop(device);
    Ok(())
}