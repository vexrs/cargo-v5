use anyhow::Result;
use clap::{Parser, Subcommand};


/// Cargo v5 accepts only individual commands and no other functions
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

   

    Ok(())
}