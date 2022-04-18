use anyhow::Result;
use clap::{Parser, Subcommand};
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
}





fn main() -> Result<()>{
    
    // Parse arguments
    let args = Args::parse();

    // Find and prepare the raw device to use
    let device = util::find_devices()?;
    let (system, user) = util::prepare_device(device)?;
    
    // Create the wrapper
    let mut device = VexDevice::new(system, user)?;

    // Match which command to use
    match args.command {
        Commands::Terminal {} => {
            println!("Not Implemented");
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
    }

    Ok(())
}