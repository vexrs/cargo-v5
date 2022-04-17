use anyhow::Result;
use clap::{Parser, Subcommand};


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


    let device = util::find_devices();
    
    println!("{:?}", device);
    


    Ok(())
}