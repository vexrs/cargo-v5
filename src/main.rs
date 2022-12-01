mod cargo_toml;
mod device_commands;
mod file;

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;


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
    let cargo_toml = cargo_toml::parse_cargo_toml();

    // Find all vex devices
    let devices = match vexv5_serial::get_socket_info_pairs() {
        Ok(v) => v,
        Err(e) => {
            print!("{} ", style("Error:").red().bright());
            println!("{}", style("Error discovering vex devices:").black().bright());
            return Err(e.into());
        }
    };

    // If there are no devices, then exit
    if devices.len() == 0 {
        print!("{} ", style("Error:").red().bright());
        println!("{}", style("No Vex devices found.").black().bright());
        // No error here because error was already printed
        return Ok(());
    }

    // If there is more than one, prompt for which one should be used
    let device = if devices.len() > 1 {
        &devices[dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .items(&devices.iter().map(|v| {
                match v {
                    vexv5_serial::SocketInfoPairs::UserSystem(u, s) => {
                        format!("{} {} {} {} {} {}", 
                            style("V5 Brain").blue().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(s.port_info.port_name.as_str()).yellow().bright(),
                            style("User:").green().bright(),
                            style(u.port_info.port_name.as_str()).yellow().bright(),
                        ).to_string()
                    },
                    vexv5_serial::SocketInfoPairs::Controller(s) => {
                        format!("{} {} {} {}", 
                            style("V5 Controller").blue().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(s.port_info.port_name.as_str()).yellow().bright(),
                        ).to_string()
                    },
                    vexv5_serial::SocketInfoPairs::SystemOnly(s) => {
                        format!("{} {} {} {}", 
                            style("Unknown VEX Device").red().bold(),
                            style("-").black().bright(),
                            style("System:").green().bright(),
                            style(s.port_info.port_name.as_str()).yellow().bright(),
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
    let device = vexv5_serial::open_device(device)?;

    // Wrap the serial port with a device structure
    let mut device = vexv5_serial::Device::new(device.0, device.1);

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
        Commands::CargoHook { file } => todo!(),
        Commands::DeviceInfo => {
            device_commands::device_info(&mut device)?;
        },
    }


    Ok(())    
}