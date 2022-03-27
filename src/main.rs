use vex_v5_serial::v5::{discover_v5_ports, protocol::VexProtocolWrapper, device::VexV5Device};
use anyhow::Result;
use clap::{Parser, Subcommand};

mod upload;

#[derive(Parser)]
#[clap(name = "v5")]
struct V5 {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Uploads this program to the v5
    #[clap()]
    Upload {
        #[clap(short = 'r', long = "run")]
        run: bool,
        slot: Option<u8>,
    },
}



fn main() -> Result<()>{
    // Just use the first vex port we find.
    let ports = discover_v5_ports()?;
    let port = ports[0].clone();

    // Open it
    let port = serialport::new(&port.port_name, 115200)
        .parity(serialport::Parity::None)
        .timeout(std::time::Duration::new(5, 0))
        .stop_bits(serialport::StopBits::One).open()?;
    

    // Create a protocol wrapper
    let wrapper = VexProtocolWrapper::new(port);

    // And create a device to use with it
    let device = VexV5Device::new(wrapper);

    let args: Vec<String> = std::env::args().collect();

    // If argument 1 is cargo then remove it
    let args = if args[1] == "v5" {
        args[1..].to_vec()
    } else {
        args
    };

    // Parse the args
    let args = V5::parse_from(args);

    // Match on subcommand
    match args.command {
        Commands::Upload { run, slot } => {
            let slot = slot.unwrap_or(1);

            upload::upload(device, slot, run)?;
        }
    }

    /*
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
    */
    Ok(())
}