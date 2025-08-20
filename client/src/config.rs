use serde::Deserialize;
use std::path::Path;
use std::error::Error;
use std::fs;


#[derive(Deserialize, Clone)]
pub struct ClientConfig {
    pub phishing_server: PhishingServer,
    pub smtp_server: SmtpServer,
    pub email_sending: EmailSending,
}

#[derive(Deserialize, Clone)]
pub struct SmtpServer{
    pub server: String,
    pub username: String,
    pub use_default_smtp_port: bool,
    pub force_smtp_port: u16
}

#[derive(Deserialize, Clone)]
pub struct PhishingServer{
    pub ip_or_domain: String, 
    pub port: u16, 
    pub is_ssl: bool,
}

#[derive(Deserialize, Clone)]
pub struct EmailSending{
    pub subject: String,
    pub from_email: String,
    pub email_template: String, 
    pub interval: u64, // 发送间隔（秒）
    pub use_appendix: bool,
    pub original_appendix_path: String,
    pub appendix_name_for_sending: String, 
}

pub fn read_config(config_path: &str) -> Result<ClientConfig,  Box<dyn Error>>{

    if !Path::new(config_path).exists() {
        let message = format!("找不到配置文件 {}", config_path);
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, message)));

    }

    let config: ClientConfig = toml::from_str(&fs::read_to_string(config_path)?)?;

    Ok(config)

}
