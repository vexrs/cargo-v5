use vex_v5_serial::v5::device::VexV5Device;
use std::path::Path;
use anyhow::Result;


pub fn upload(device: VexV5Device<T>, slot: u8, run: bool) -> Result<()> {
    // Try to find a Cargo.toml in the current directory
    let cargo = Path::new("./Cargo.toml");

    // If we can't find it, then we can't upload
    if !cargo.exists() {
        return Err(anyhow::anyhow!("Could not find Cargo.toml in the current directory"));
    }

    // Parse the toml file


    Ok(())
}