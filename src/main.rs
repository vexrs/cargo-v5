use vex_v5_serial::v5::protocol::vex::{VexDeviceType, VexProtocolWrapper, VexDeviceCommand};
use anyhow::Result;

fn main() -> Result<()>{
    let port = serialport::new("/dev/ttyACM0", 115200).open()?;

    let mut wrapper = VexProtocolWrapper::new(VexDeviceType::System, port);

    let to_serialize: (u8, u8, [u8; 24]) = (1, 0, *b"slot_1.bin\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    let data = bincode::serialize(&to_serialize)?;
    wrapper.send_extended(VexDeviceCommand::ExecuteFile, data)?;
    

    drop(wrapper);
    Ok(())
}