use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::string::{String, ToString};
use std::vec::Vec;
use serde::Deserialize;
use rand::{thread_rng, Rng};
extern crate embed_resource;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub appendix: AppendixConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppendixConfig{
    pub ip_or_domain: String,
    pub port: u16,
    pub is_rc: bool,
    pub rc_path: Option<String>,
}

fn main() {
    let config: Config = read_config("../build_config.toml").expect("Failed to read configuration");

    let ip_or_domain = config.appendix.ip_or_domain;
    let port = config.appendix.port;
    let is_rc = config.appendix.is_rc;
    let rc_path = config.appendix.rc_path;

    let mut rand = thread_rng();
    let key: u16 = rand.r#gen();

    let encrypted_str = simple_encrypt(&ip_or_domain, key);

    let path = Path::new("config.rs");
    let mut file = File::create(&path).unwrap();

    writeln!(
        file,
        "
        static IP_OR_DOMAIN: Once<alloc::vec::Vec<u16>> = Once::new();

        pub fn init_ip_or_domain() {{
            let _ = IP_OR_DOMAIN.call_once(|| {{
                alloc::vec!{:?} 
            }});
        }}

        pub static PORT: u16 = {:?};
        const KEY: u16 = {};

        fn simple_decrypt(input: &[u16], key: u16) -> alloc::vec::Vec<u16> {{
            let decrypted_utf16: alloc::vec::Vec<u16> = input.iter().map(|x| x ^ key).collect();
            decrypted_utf16
        }}

        pub fn get_ip_or_domain() -> alloc::vec::Vec<u16> {{
            simple_decrypt(IP_OR_DOMAIN.get().unwrap(), KEY)
        }}
        ", encrypted_str, port, key
    ).unwrap();

    // 添加图标
    if is_rc && rc_path.is_some() {
        embed_resource::compile(rc_path.unwrap(), embed_resource::NONE).manifest_optional().unwrap();
    }

}

fn read_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config = toml::from_str(&config_str)?;
    Ok(config)
} 

fn simple_encrypt(input: &str, key: u16) -> Vec<u16> {
    let utf16: Vec<u16> = input.encode_utf16().collect();
    utf16.iter().map(|c| c ^ key).collect()
}
