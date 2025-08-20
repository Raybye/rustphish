use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub links: Links,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Links{
    pub index: String,
}

fn main() {
    let config: Config = read_config("../build_config.toml").expect("Failed to read configuration");

    let link_index = config.links.index;
    let path = Path::new("config.rs");
    let mut file = File::create(&path).unwrap();

    writeln!(
        file,
        "
        pub const LINK_INDEX: &str = {:?};
        ", link_index
    ).unwrap();

}

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config = toml::from_str(&config_str)?;
    Ok(config)
} 
