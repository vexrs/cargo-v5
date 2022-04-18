use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use util::DevicePair;
use vexv5_serial::device::VexDevice;


mod util;
mod files;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Opens a terminal connection to the v5 brain
    Terminal {},
}





fn main() -> Result<()>{
    
    // Parse arguments
    let args = Args::parse();


    let device = util::find_devices()?;

    let (system, user) = match device {
        DevicePair::Double(d1, d2) => {
            (
                (
                    d1.clone(),
                    serialport::new(d1.port_info.port_name, 115200)
                    .parity(serialport::Parity::None)
                    .timeout(Duration::new(vexv5_serial::device::SERIAL_TIMEOUT_SECONDS, vexv5_serial::device::SERIAL_TIMEOUT_NS))
                    .stop_bits(serialport::StopBits::One).open()?
                ),
                Some(
                    (
                        d2.clone(),
                        serialport::new(d2.port_info.port_name, 115200)
                            .parity(serialport::Parity::None)
                            .timeout(Duration::new(vexv5_serial::device::SERIAL_TIMEOUT_SECONDS, vexv5_serial::device::SERIAL_TIMEOUT_NS))
                            .stop_bits(serialport::StopBits::One).open()?
                    )
                ),
            )
        },
        DevicePair::Single(d1) => {
            (
                (
                    d1.clone(),
                    serialport::new(d1.port_info.port_name, 115200)
                        .parity(serialport::Parity::None)
                        .timeout(Duration::new(vexv5_serial::device::SERIAL_TIMEOUT_SECONDS, vexv5_serial::device::SERIAL_TIMEOUT_NS))
                        .stop_bits(serialport::StopBits::One).open()?
                ),
                None
            )
        }
    };

    
    let mut device = VexDevice::new(system, user)?; 
    
    device.with_channel(vexv5_serial::device::V5ControllerChannel::UPLOAD, |d| {
        let name = "test.txt";
        // Get the info of slot_1.ini
        //let metadata = d.file_metadata_from_name(name.to_string(), None, None)?;

        
        
        // Read in the data from the file
        let data = std::fs::read(name)?;

        files::upload_file(d, name.to_string(), data)?;

        
        

        Ok(())
    })?;
    
    

    Ok(())
}