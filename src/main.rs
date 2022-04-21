use std::io::{Read, Write};

use anyhow::Result;
use ascii::AsAsciiStr;
use clap::{Parser, Subcommand};
use chrono::prelude::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use vexv5_serial::device::{VexDevice, V5ControllerChannel};



use crate::util::read_cobs_packet;


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
    /// Downloads a file from the brain
    Download {
        /// The file to download
        file: String
    },
    /// Uploads a file to the brain
    Upload {
        /// The file to upload
        file: String
    },
    /// Should be used by cargo only. Generates files, uploads a program and runs it.
    CargoHook {
        /// The program file to upload
        file: String,
    }
}



#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct CargoToml {
    package: Package,
}


fn terminal<T: Read+Write>(device: &mut VexDevice<T>) -> Result<()> {
    // We want to use a download channel
    device.with_channel(V5ControllerChannel::UPLOAD, |device| {
        // Use a buffer to store data that will be read into COBS packets.
        let mut buf = Vec::<u8>::new();
        loop {
            // Read in the COBS packet.
            let decoded = read_cobs_packet(device, &mut buf)?;

            // If it starts with `sout` we know it is PROS
            // so just print it.
            if decoded.starts_with(b"sout") {
                print!("{}", decoded[4..].as_ascii_str()?);
                // Flush stdout just in case the output of the program does not contain a newline.
                std::io::stdout().flush()?;
            } else {
                // If not, print it raw
                print!("{}", decoded.as_ascii_str()?);
            }

            
        }
    })?;
    Ok(())
}

fn main() -> Result<()>{
    
    let args: Vec<String> = std::env::args().collect();

    // If argument 1 is cargo then remove it
    let args = if args.len() < 2 {
        args
    } else if args[1] == "v5" {
        args[1..].to_vec()
    } else {
        args
    };

    // Parse the args
    let args = Args::parse_from(args);

    // Find and prepare the raw device to use
    let device = util::find_devices()?;
    let (system, user) = util::prepare_device(device)?;
    
    // Create the wrapper
    let mut device = VexDevice::new(system, user)?;

    // Match which command to use
    match args.command {
        Commands::Terminal {} => {
            // Constantly read and print data
            terminal(&mut device)?;
            
            
        },
        Commands::Download { file } => {
            // Download the file
            let data = files::download_file(&mut device, file.clone())?;

            // Write the file to disk
            std::fs::write(file, data)?;
        },
        Commands::Upload { file } => {
            // Read the data from disk
            let data = std::fs::read(file.clone())?;

            // Upload the file
            files::upload_file(&mut device, file, data)?;
        },
        Commands::CargoHook { file } => {


            // Objcopy the file to a .bin file
            // We expect arm-none-eabi-objcopy
            // TODO: A good idea would be to implement an objcopy alternative in rust.
            let mut command = std::process::Command::new("arm-none-eabi-objcopy");
            command.arg("-O").arg("binary");
            command.arg(file.clone());

            // Add the bin prefix
            let mut upload_file = file;
            upload_file.push_str(".bin");
            command.arg(upload_file.clone());

            // Run the command
            command.output()?;

            // Detect if the slot file exists
            let slot_file = std::path::Path::new("slot");
            if !slot_file.exists() {
                // If it doesn't exist, create it, defaulting the slot number to 0
                std::fs::write(slot_file, "0")?;
            }

            // Read in the slot file, parsing its contents into an u8
            let slot: u8 = std::fs::read_to_string(slot_file)?.parse()?;

            // Try to find a Cargo.toml in the current directory
            let cargo = std::path::Path::new("./Cargo.toml");

            // If we can't find it, then we can't upload
            if !cargo.exists() {
                return Err(anyhow::anyhow!("Could not find Cargo.toml in the current directory"));
            }

            // Parse the toml file
            let f = std::fs::read_to_string(cargo)?;
            let parsed_toml = toml::from_str::<CargoToml>(&f)?;

            // Get the current time and format it as ISO 8601
            let time = std::time::SystemTime::now();
            let time = <DateTime<Utc>>::from(time).format("%+");

            // Create the ini file
            let mut ini = Vec::<String>::new();
            ini.push("[program]".to_string());
            ini.push(format!("name = \"{}\"", parsed_toml.package.name));
            ini.push(format!("version = \"{}\"", parsed_toml.package.version));
            ini.push(format!("description = \"{}\"", parsed_toml.package.description.unwrap_or_else(|| "".to_string())));
            ini.push(format!("slot = {}", slot));
            ini.push(format!("date = {}", time));


            // We use the Vex X logo just because.
            ini.push("icon = USER001x.bmp".to_string());

            // Join into a single string
            let ini = ini.join("\n");

            // Convert to an ascii string
            let ini = ini.as_ascii_str()?;
            // Convert to bytes
            let ini: Vec<u8> = ini.as_bytes().to_vec();

            // Upload the file
            files::upload_file(&mut device, format!("slot_{}.ini", slot+1), ini)?;

            // Read in the file to upload
            let data = std::fs::read(upload_file)?;

            // Upload it to the brain
            files::upload_file(&mut device, format!("slot_{}.bin", slot+1), data)?;

            // Run the program file
            device.execute_program_file(format!("slot_{}.bin", slot+1), None, None)?;

            // Open terminal
            terminal(&mut device)?;
        }
    }

    Ok(())
}