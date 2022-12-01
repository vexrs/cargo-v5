//! Contains structs that will parse the current project's cargo.toml if it exists

use anyhow::Result;
use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CargoToml {
    pub package: Package,
}

impl Default for CargoToml {
    fn default() -> CargoToml {
        CargoToml {
            package: Package {
                name: "Default Project Name".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Default Project Description. Your project may be missing a Cargo.toml, or you are not in your project's root directory.".to_string()),
            }
        }
    }
}

pub fn parse_cargo_toml() -> Result<CargoToml> {
    // Try to find the Cargo.toml in the current directory
    let cargo = std::path::Path::new("./Cargo.toml");

    // If it does not exist, then just return a default CargoToml
    if !cargo.exists() {
        return Ok(CargoToml::default());
    }

    // Load and parse the cargo.toml file
    let f = std::fs::read_to_string(cargo)?;
    let parsed_toml = toml::from_str::<CargoToml>(&f)?;

    // Return the parsed toml
    Ok(parsed_toml)
}