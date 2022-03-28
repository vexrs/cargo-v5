use anyhow::Result;


fn main() -> Result<()> {
    // Get the directory to place data in
    let data_dir = dirs::home_dir().unwrap();
    let data_dir = data_dir.join(".v5");

    // If it doesn't exist, create it
    if !data_dir.exists() {
        std::fs::create_dir(&data_dir)?;
    }

    // If the .v5init does not exist in the data directory
    // we need to install the required libraries
    let init_file = data_dir.join(".v5init");
    if !init_file.exists() {
        // Get the LIBV5_PATH environment variable
        let libv5_path = std::env::var("LIBV5_PATH");

        // If it is none, we can not upload
        if libv5_path.is_err() {
            return Err(anyhow::anyhow!("Could not find LIBV5_PATH environment variable"));
        }

        let libv5_path = libv5_path.unwrap();

        // Make the path to the libv5_dir
        let libv5_dir = data_dir.join("libv5rt");
        
        // If it does not exist, create it
        if !libv5_dir.exists() {
            std::fs::create_dir_all(&libv5_dir)?;
        }

        // Copy all files from libv5_path to the data directory
        let mut co = fs_extra::dir::CopyOptions::new();
        co.content_only = true;
        fs_extra::dir::copy(libv5_path, libv5_dir, &co)?;

        // Create the .v5init file
        std::fs::write(&init_file, "")?;
    }

    Ok(())
}