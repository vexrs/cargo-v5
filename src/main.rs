use std::time::Duration;

use anyhow::Result;
use clap::{Parser, Subcommand};
use util::DevicePair;
use vexv5_serial::{protocol::V5Protocol, device::VexDevice};
use spinners::{Spinner, Spinners};

mod util;


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

        // Write to the slot_1.ini file on the brain
        let mut fh = d.open(name.to_string(), Some(vexv5_serial::device::VexInitialFileMetadata {
            function: vexv5_serial::device::VexFileMode::Upload(vexv5_serial::device::VexFileTarget::FLASH, true),
            vid: vexv5_serial::device::VexVID::USER,
            options: 0,
            length: data.len() as u32,
            addr: 0x3800000,
            crc: crc::Crc::<u32>::new(&vexv5_serial::protocol::VEX_CRC32).checksum(&data),
            r#type: *b"bin\0",
            timestamp: 0,
            version: 0x01000000,
            linked_name: None,
        }))?;

        

        // Write data
        util::write_file_progress(&mut fh, data)?;
        
        // We are doing a file transfer, so it may take some time for the final response.
        // Just increase the timeout here
        d.set_timeout(Some(Duration::new(15, 0)));

        // We will also setup a spinner so the user knows that the application has not frozen.
        let sp = Spinner::new(Spinners::Dots, "Closing File Handle".to_string());

        // Close file
        fh.close(vexv5_serial::device::VexFiletransferFinished::ShowRunScreen)?;
        
        // And stop the spinner
        sp.stop();

        // Reset the timeout to default
        d.set_timeout(None);

        

        Ok(())
    })?;
    
    // Add an extra newline to avoid some weirdness
    print!("\n");

    Ok(())
}