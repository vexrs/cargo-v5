mod cargo_toml;
mod device_commands;
mod file;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;

use chrono::prelude::{DateTime, Utc};
use vexv5_serial::devices::VexDeviceType;


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Opens a terminal connection to the v5 brain
    Terminal,
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
    /// Prints device info
    DeviceInfo,
    /// Should be used by cargo only. Generates files, uploads a program and runs it.
    CargoHook {
        /// The program file to upload
        file: String,
    }
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
    let args = match Args::try_parse_from(args) {
        Ok(v) => v,
        Err(e) => {
            // If this fails, it will print help and gracefully exit
            print!("{}", e);
            return Ok(());
        }
    };

    // Load the Cargo.toml file
    let cargo_file = cargo_toml::parse_cargo_toml()?;

    // Find all vex devices
    let devices = vexv5_serial::devices::genericv5::find_generic_devices()?;

    // If there is more than one, prompt for which one should be used
    let device = if devices.len() > 1 {
        &devices[dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&devices.iter().map(|v| {
                match v.device_type {
                    VexDeviceType::Brain => {
                        format!("{} {} {} {} {} {}", 
                            style("V5 Brain").blue().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(v.system_port.clone()).yellow().bright(),
                            style("User:").green().bright(),
                            style(v.user_port.as_ref().unwrap().clone()).yellow().bright(),
                        ).to_string()
                    },
                    VexDeviceType::Controller => {
                        format!("{} {} {} {}", 
                            style("V5 Controller").blue().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(v.system_port.clone()).yellow().bright(),
                        ).to_string()
                    },
                    VexDeviceType::Unknown => {
                        format!("{} {} {} {}", 
                            style("Unknown VEX Device").red().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(v.system_port.clone()).yellow().bright(),
                        ).to_string()
                    },
                    
                }
            }).collect::<Vec<String>>())
            .default(0)
            .interact()?]
    } else {
        &devices[0]
    };

    // Open the vex device
    let mut device = device.open()?;

    // Run the proper commands
    match args.command {
        Commands::Terminal => {
            device_commands::terminal(&mut device)?;
        },
        Commands::Download { file } => {
            // Download the file
            let data = file::download_file(&mut device, file.clone(), vexv5_serial::file::FTComplete::DoNothing)?;

            // Write the file to disk
            std::fs::write(file, data)?;
        },
        Commands::Upload { file } => {
            // Read the data from disk
            let data = std::fs::read(file.clone())?;

            // Upload the file
            file::upload_file(&mut device, file, data, vexv5_serial::file::FTComplete::DoNothing)?;
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
            let parsed_toml = cargo_file;

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

            // Convert to bytes
            let ini: Vec<u8> = ini.as_bytes().to_vec();

            // Read in the bin file to upload
            let data = std::fs::read(upload_file)?;

            // Upload it to the brain
            file::upload_file(&mut device, format!("slot_{}.bin", slot+1), data, vexv5_serial::file::FTComplete::RunProgram)?;

            
            // Upload the ini file
            file::upload_file(&mut device, format!("slot_{}.ini", slot+1), ini, vexv5_serial::file::FTComplete::DoNothing)?;

            // Open terminal
            device_commands::terminal(&mut device)?;

           
        },
        Commands::DeviceInfo => {
            device_commands::device_info(&mut device)?;
        },
    }


    Ok(())    
}