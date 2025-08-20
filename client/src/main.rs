use clap::{Arg, Command};
use colored::*;
use std::error::Error;
use std::fs::{self, create_dir_all};
use std::path::Path;
use std::thread;
use std::time::Duration;
use std::sync::Arc;

#[cfg(feature = "mail")]
use rpassword::read_password;

#[cfg(feature = "mail")]
mod smtp;

#[cfg(feature = "mail")]
pub mod generator;

#[cfg(feature = "db")]
mod db;

#[cfg(feature = "db")]
use db::EmailEntry;

#[cfg(feature = "appendix")]
pub mod malware;

#[cfg(feature = "qrcode")]
pub mod qr;

mod config;
use config::*;

const BANNER: &str = r#"
██████╗ ██╗   ██╗███████╗████████╗██████╗ ██╗  ██╗██╗███████╗██╗  ██╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔══██╗██║  ██║██║██╔════╝██║  ██║
██████╔╝██║   ██║███████╗   ██║   ██████╔╝███████║██║███████╗███████║
██╔══██╗██║   ██║╚════██║   ██║   ██╔═══╝ ██╔══██║██║╚════██║██╔══██║
██║  ██║╚██████╔╝███████║   ██║   ██║     ██║  ██║██║███████║██║  ██║
╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝     ╚═╝  ╚═╝╚═╝╚══════╝╚═╝  ╚═╝
                            [Made by Ky9oss (https://github.com/Ky9oss)]
"#;

pub fn print_status(status: &str, message: &str) {
    println!("[{}] {}", status.bold(), message);
}

pub fn print_success(message: &str) {
    print_status(&"✓".green(), message);
}

pub fn print_error(message: &str) {
    print_status(&"✗".red(), message);
}

pub fn print_info(message: &str) {
    print_status(&"i".blue(), message);
}


#[cfg(feature = "mail")]
async fn send_phishing_emails(
    emails: Vec<EmailEntry>,
    config: ClientConfig,
    password: String,
) -> Result<(), Box<dyn Error>> {
    // 检查配置文件
    if !Path::new(&config.email_sending.email_template).exists() {
        let message = format!("找不到邮件模板文件 {}", config.email_sending.email_template);
        print_error(&message);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            message,
        )))
    };

    print_info(&format!("找到 {} 个目标邮箱", emails.len()));

    // 读取邮件模板
    let template = fs::read_to_string(&config.email_sending.email_template)?;

    // 验证SMTP凭证
    print_info("验证SMTP凭证...");
    smtp::verify_smtp_credentials(config.smtp_server.clone(), &password)?;
    print_success("SMTP凭证验证成功");

    let sleep = config.email_sending.interval;
    let config = Arc::new(config);
    let password = Arc::new(password.clone());
    let template = Arc::new(template);

    // 发送邮件
    for entry in emails {
        let config_clone = Arc::clone(&config);
        let password = Arc::clone(&password);
        let template = Arc::clone(&template);

        std::thread::spawn(move || {
            let smtp_server: &SmtpServer= &config_clone.smtp_server;
            let phishing_server: &PhishingServer= &config_clone.phishing_server;
            let email_sending: &EmailSending= &config_clone.email_sending;

            print_info(&format!("正在发送邮件到 {}", entry.email));
            let temp_file = generator::generate_tmp_email(template.to_string(), entry.clone(), phishing_server).unwrap();

            match smtp::send_html_email(
                smtp_server,
                email_sending,
                &password,
                entry.clone(),
                &temp_file,
            ) {
                Ok(_) => print_success(&format!("发送成功: {}", entry.email)),
                Err(e) => print_error(&format!("发送失败 {}: {}", entry.email, e)),
            }

            // 删除临时文件
            fs::remove_file(&temp_file).unwrap_or(());
        });

        // 等待指定时间间隔
        thread::sleep(Duration::from_secs(sleep));
    }


    thread::sleep(Duration::from_secs(5));

    // 清理临时目录
    remove_temp_dir("./temp")?;
    print_success("程序执行完成，临时文件已成功清除。");

    Ok(())
}


fn remove_temp_dir(temp_path: &str) -> Result<(), Box<dyn Error>> {

    if fs::metadata(temp_path).is_ok() {
        fs::remove_dir_all(temp_path)?;  // 递归删除整个目录及子目录
    }

    Ok(())
}

fn show_all_emails(email_tree: &sled::Tree) -> Result<(), Box<dyn Error>> {
    let emails = db::get_all_emails(email_tree)?;

    if emails.is_empty() {
        print_info("数据库中没有邮箱记录");
        return Ok(());
    }

    let length = emails.len();

    println!("\n{:=^60}", " 邮箱列表 ");
    println!("{:<8} {:<15} {}", "序号", "ID", "邮箱地址");
    println!("{:-<60}", "");

    // 添加计数器，生成从1开始的连续序号
    let mut counter = 1;
    for entry in emails {
        println!("{:<8} {:<15} {}", counter, entry.id, entry.email);
        counter += 1;
    }
    println!("{:=^60}\n", "");

    print_success(&format!("共找到 {} 个邮箱", length));

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", BANNER.bright_cyan());

    #[cfg(feature = "db")]
    const DB_PATH: &str = "email_database";
    #[cfg(feature = "db")]
    const EMAIL_TREE: &str = "emails";

    let mut app = Command::new("Rustphish Client")
        .version("1.3")
        .author("Ky9oss")
        .about("轻量级邮件钓鱼工具")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG_FILEPATH")
                .help("指定配置文件路径（默认./client_config.toml）"),
        );


    #[cfg(feature = "db")]
    {
        app = app
            .arg(
                Arg::new("read")
                    .short('r')
                    .long("read")
                    .value_name("DATABASE")
                    .help("读取并格式化显示钓鱼记录"),
            )
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .value_name("EMAIL_LIST")
                    .help("从文件导入目标邮箱列表"),
            )
            .arg(
                Arg::new("show")
                    .long("show")
                    .help("显示所有目标邮箱")
                    .num_args(0),
            )
            .arg(
                Arg::new("delete")
                    .short('d')
                    .long("delete")
                    .value_name("ID")
                    .help("删除指定ID的邮箱记录"),
            );
    }

    #[cfg(feature = "mail")]
    {
        app = app
            .arg(
                Arg::new("send-all")
                    .long("send-all")
                    .help("向所有目标发送钓鱼邮件")
                    .num_args(0),
            )
            .arg(
                Arg::new("send")
                    .long("send")
                    .value_name("ID")
                    .help("向指定ID的目标发送钓鱼邮件"),
            )
            .arg(
                Arg::new("send-from-to")
                    .long("send-from-to")
                    .value_name("ft")
                    .help("发送邮件区间"),
            )
    }

    let matches = app.clone().get_matches();

    // 如果没有匹配任何命令
    if !matches.args_present() {
        app.print_help().unwrap();
        return Ok(())
    }

    let config_path: &str = match matches.get_one::<String>("config") {
        Some(config_path) => {
            config_path
        },
        None => {
            let str: &str = "client_config.toml";
            str
        }
    };

    let config: ClientConfig = match read_config(config_path) {
        Ok(config) => {
            config
        },
        Err(e) => {
            print_error(&format!("读取配置文件错误： {:?}", e));
            return Ok(())
        }
    };


    // 数据库相关功能
    #[cfg(feature = "db")]
    {
        let db = sled::open(DB_PATH)?;
        let email_tree = db.open_tree(EMAIL_TREE)?;

        if let Some(db_path) = matches.get_one::<String>("read") {
            print_info(&format!("正在读取数据库: {}", db_path));

            let db_server = match sled::open(db_path) {
                Ok(db) => {
                    print_success("服务器数据库打开成功");
                    db
                }
                Err(e) => {
                    print_error(&format!("打开服务器数据库失败: {}", e));
                    return Err(Box::new(e));
                }
            };

            let action_tree = match db_server.open_tree("actions") {
                Ok(tree) => tree,
                Err(e) => {
                    print_error(&format!("打开actions表失败: {}", e));
                    return Err(Box::new(e));
                }
            };

            let data_tree = match db_server.open_tree("data") {
                Ok(tree) => tree,
                Err(e) => {
                    print_error(&format!("打开data表失败: {}", e));
                    return Err(Box::new(e));
                }
            };

            match db::traverse_actions(&action_tree, &data_tree, &email_tree) {
                Ok(_) => (),
                Err(e) => {
                    print_error(&format!("遍历钓鱼记录失败: {}", e));
                    return Err(e);
                }
            };
        } else if let Some(input_path) = matches.get_one::<String>("input") {
            print_info(&format!("正在导入邮箱列表: {}", input_path));
            match db::load_emails_to_db(&email_tree, input_path) {
                Ok(_) => print_success("邮箱列表导入成功"),
                Err(e) => print_error(&format!("导入失败: {}", e)),
            }
        } else if matches.get_flag("show") {
            show_all_emails(&email_tree)?;
        } else if let Some(id) = matches.get_one::<String>("delete") {
            if let Err(e) = db::delete_email_by_id(&email_tree, id) {
                print_error(&format!("删除失败: {}", e));
            }
        }
    }

    // 邮件相关功能
    #[cfg(feature = "mail")]
    {
        let db = sled::open(DB_PATH)?;
        let email_tree = db.open_tree(EMAIL_TREE)?;

        let rt = tokio::runtime::Runtime::new()?;

        if matches.get_flag("send-all") {
            print_info("请输入SMTP密码：");
            let password = read_password()?;
            print_info("开始批量发送邮件");
            rt.block_on(async {
                    let emails = db::get_all_emails(&email_tree)?;
                    send_phishing_emails(emails, config, password).await?;
                Ok::<(), Box<dyn Error>>(())
            })?;
            print_success("所有邮件发送完成");
        } else if let Some(target_id) = matches.get_one::<String>("send") {
            print_info("请输入SMTP密码：");
            let password = read_password()?;
            print_info("开始批量发送邮件");
            rt.block_on(async {
                match email_tree.get(target_id.as_bytes())? {
                    Some(value) => {
                        let entry: EmailEntry = bincode::deserialize(&value)?;
                        let vec_entry: Vec<EmailEntry> = vec!(entry);
                        send_phishing_emails(vec_entry, config, password).await?;
                    },
                    None => {
                    }
                };
                Ok::<(), Box<dyn Error>>(())
            })?;

        } else if let Some(ft) = matches.get_one::<String>("send-from-to") {
            print_info("请输入SMTP密码：");
            let password = read_password()?;
            let from_to: Vec<u16> = ft.split('-').map(|x| x.parse().unwrap()).collect();
            let from = from_to[0];
            let to = from_to[1];

            print_info("开始批量发送邮件");
            rt.block_on(async {
                    let emails = db::get_all_emails(&email_tree)?;
                    let end = std::cmp::min(to, emails.len().try_into().unwrap());
                    let new_emails = emails[from as usize..end as usize].to_vec();
                    send_phishing_emails(new_emails, config, password).await?;
                    Ok::<(), Box<dyn Error>>(())
            })?;
        } 
    }


    Ok(())
}
