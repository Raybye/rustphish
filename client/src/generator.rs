use crate::config::*;
use crate::db::EmailEntry;
use std::error::Error;
use std::fs::{self, create_dir_all};
use std::path::Path;

#[cfg(feature = "qrcode")]
use crate::qr;

include!("../config.rs");

fn replace_id(id_value: &str) -> String {
    LINK_INDEX.replace("{id}", id_value)
}

pub fn generate_tmp_email(template: String, entry: EmailEntry, phishing_server: &PhishingServer) -> Result<String, Box<dyn Error>>{

    let index_path = replace_id(&entry.id);

    let index_url = match phishing_server.is_ssl {
        true => format!("https://{}:{}{}", &phishing_server.ip_or_domain, &phishing_server.port, &index_path),
        false => format!("http://{}:{}{}", &phishing_server.ip_or_domain, &phishing_server.port, &index_path),
    };
    let image_url = match phishing_server.is_ssl {
        true => format!("https://{}:{}/image/{}", &phishing_server.ip_or_domain, &phishing_server.port, &entry.id),
        false => format!("http://{}:{}/image/{}", &phishing_server.ip_or_domain, &phishing_server.port, &entry.id),
    };

    let content = template.replace("{{index}}", &index_url);
    let content = content.replace("{{image}}", &image_url);

    #[cfg(feature = "qrcode")]
    let content = if content.contains("{{qrcode}}") {
        let qrimg = qr::generate_qrcode_html(&index_url)?;
        let content = content.replace("{{qrcode}}", &qrimg);
        content
    } else {
        content
    };

    #[cfg(not(feature = "qrcode"))]
    let content = if content.contains("{{qrcode}}") {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "你的程序没有编译qrcode功能，不支持二维码钓鱼，请修改你的配置文件或重新编译"
        )))
    } else {
        content
    };

    // 创建临时文件存储当前邮件内容
    let temp_dir = Path::new("./temp");
    create_dir_all(temp_dir)?;
    let temp_file = format!("temp/{}.html", entry.id);
    fs::write(&temp_file, &content)?;

    Ok(temp_file)

}
