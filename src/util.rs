use std::{io::{Read, Write}};

use serialport::SerialPortType;
use vexv5_serial::{ports::{VexSerialInfo, VexSerialClass}, device::V5FileHandle};
use console::style;
use dialoguer::{
    Select,
    theme::ColorfulTheme
};
use anyhow::Result;


#[derive(Clone, Debug)]
pub enum DevicePair {
    Single(VexSerialInfo),
    Double(VexSerialInfo, VexSerialInfo)
}


pub fn find_devices() -> Result<DevicePair> {
    // Try to find vex devices
    let devices = vexv5_serial::ports::discover_vex_ports()?;

    // Create an empty vector with device pairs
    let mut pairs = Vec::<DevicePair>::new();

    // This mimics behavior of PROS assuming that the second device is always the user device.
    for device in devices {
        match device.class {
            VexSerialClass::User => {
                if pairs.len() > 0 {
                    
                    if let DevicePair::Single(d) = pairs.last().unwrap().clone() {
                        pairs.pop();
                        pairs.push(DevicePair::Double(d.clone(), device));
                    } else {
                        pairs.push(DevicePair::Single(device));
                    }
                } else {
                    pairs.push(DevicePair::Single(device));
                }
            },
            _ => {
                pairs.push(DevicePair::Single(device));
            },
            
        }
    }
    
    // If there are no devices, then error
    if pairs.len() == 0 {
        print!("{} ", style("Error:").red().bright());
        println!("{}", style("No Vex devices found.").black().bright());
        return Err(anyhow::anyhow!("No Devices Found"));
    }

    // If there is only one device, then use it.
    // If not, then ask which one to use
    let device = if pairs.len() == 1 {
        pairs[0].clone()
    } else {
        let device = pairs[0].clone();

        // Generate a list of selections (just differently formatted devices)
        let mut pselect = Vec::<String>::new();
        for pair in pairs.clone() {
            if let DevicePair::Single(d1) = pair {
                pselect.push(format!("{:?} port: {} ({})", d1.class, d1.port_info.port_name, match d1.port_info.port_type {
                    SerialPortType::UsbPort(p) => {
                        p.product.unwrap_or("".to_string())
                    },
                    _ => {
                        "Unsupported Device".to_string()
                    }
                }));
            } else if let DevicePair::Double(d1, d2) = pair {
                pselect.push(format!("Vex Brain with ports {} and {}",
                    d1.port_info.port_name,
                    d2.port_info.port_name
                ));
            }
            
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&pselect)
            .default(0)
            .with_prompt("Multiple Vex devices found. Please select which one you want to use:")
            .interact()?;

        pairs[selection].clone()
    };

    Ok(device)
}


pub fn write_file_progress<T: Read + Write>(handle: V5FileHandle<T>) -> Result<()> {

    

    Ok(())
}