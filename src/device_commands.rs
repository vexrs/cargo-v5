use std::io::{Write, Read};

use console::style;
use vexv5_serial::{Device, remote::{SwitchChannel, V5ControllerChannel}};
use anyhow::Result;

/// Uses a specific controller channel while inside of the provided closure
/// Acts as a context manager
pub fn with_channel<U, S, R, F>(device: &mut Device<U, S>, channel: V5ControllerChannel, f: F) -> Result<R>
where U: Read+Write, S: Read + Write, F: Fn(&mut Device<U, S>) -> Result<R> {
    // Switch to the selcted channel if this device is a controller
    if device.is_controller()? {
        device.send_request(SwitchChannel(channel))?;
    }

    // Run the closude
    let r = f(device);

    // Switch back to pit channel if this device is a controller
    if device.is_controller()? {
        device.send_request(SwitchChannel(V5ControllerChannel::Pit))?;
    }
    
    // Return the closure's result
    r
}

pub fn device_info<S: Read+Write, U: Read+Write>(device: &mut vexv5_serial::Device<S, U>) -> anyhow::Result<()> {
    // Get the vex device system info
    let info = device.send_request(vexv5_serial::system::GetSystemVersion())?;

    // Pretty print the info
    match info.product_type {
        vexv5_serial::system::VexProductType::V5Brain(_flags) => {
            println!("{}", style("V5 Brain").red());
        },
        vexv5_serial::system::VexProductType::V5Controller(flags) => {

            let out = if flags.contains(vexv5_serial::system::V5ControllerFlags::CONNECTED_CABLE) {
                "Tethered".to_string()
            } else if flags.contains(vexv5_serial::system::V5ControllerFlags::CONNECTED_WIRELESS) {
                "Connected".to_string()
            } else {
                "Disconnected".to_string()
            };

            println!("{} - {}",
                style("V5 Controller").red(),
                style(out).black().bright(),
            );
        },
    }
    println!("{} {}.{}.{}.{}",
        style("System Version").blue().dim(),
        style(info.system_version.0).blue(),
        style(info.system_version.1).blue(),
        style(info.system_version.2).blue(),
        style(info.system_version.3).blue(),
    );

    Ok(())
}

pub fn terminal<U: Read+Write, S: Read+Write>(device: &mut Device<U, S>) -> Result<()> {

    // Use the download channel if this is a controller
    with_channel(device, V5ControllerChannel::Download, |d| -> Result<()> {

        // Use VexrsSerial
        let mut serial = vexrs_serial::protocol::VexrsSerial::new(d);

        // Loop forever
        loop {
            // Read in data
            let data = serial.read_data()?;

            // If it should be printed, then print it and flush stdout
            if let vexrs_serial::data::DataType::Print(d) = data {
                print!("{}", std::str::from_utf8(&d)?);
                std::io::stdout().flush()?;
            }

        }
    })?;

    Ok(())
}