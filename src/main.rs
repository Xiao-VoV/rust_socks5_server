// src/main.rs
use clap::Parser;
use std::error::Error;
use tokio::net::TcpListener;
use tracing::{Level, error, info};

mod auth;
mod consts;
mod handler;
mod protocol;
mod udp;

use auth::UserConfig;

use crate::auth::User;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 监听地址
    #[arg(short, long, default_value = "127.0.0.1")]
    ip: String,

    /// 监听端口
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// 认证用户名 (可选)
    #[arg(short, long)]
    user: Option<String>,

    /// 认证密码 (可选，必须配合 user 使用)
    #[arg(long)]
    pass: Option<String>,

    /// 超时时间
    #[arg(long, default_value_t = 5)]
    timeout: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .init();

    let args = Args::parse();

    let mut config = UserConfig {
        user: None,
        timeout: args.timeout,
    };
    let timeout = args.timeout;
    if let Some(user) = args.user {
        if let Some(pass) = args.pass {
            info!("use auth,user:{}", user);
            config = UserConfig {
                user: Some(User {
                    username: user,
                    password: pass,
                }),
                timeout,
            };
        } else {
            error!("no password");
            std::process::exit(1);
        }
    } else {
        info!("running in No_auth");
    }

    let config = std::sync::Arc::new(config);

    let addr = format!("{}:{}", args.ip, args.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("SOCKS5 Server running on {}", addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        let config_clone = config.clone();

        tokio::spawn(async move {
            if let Err(e) = handler::process(socket, config_clone.as_ref()).await {
                error!("[Error] from {:?} : {}", addr, e);
            }
        });
    }
}
