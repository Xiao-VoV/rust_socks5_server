// src/auth.rs
use crate::consts::*;
use std::error::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
}
// 简单的用户配置结构
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub user: Option<User>,
    pub timeout: u8,
}

pub async fn perform_password_auth(
    socket: &mut TcpStream,
    user_config: &User,
) -> Result<(), Box<dyn Error>> {
    // 1. 读取版本号和用户名长度 [VER, ULEN]
    let mut header = [0u8; 2];
    socket.read_exact(&mut header).await?;

    let ver = header[0];
    let ulen = header[1] as usize;

    if ver != AUTH_VERSION {
        return Err(format!("unsupport auth version: {}", ver).into());
    }

    // 2. 读取用户名
    let mut user_buf = vec![0u8; ulen];
    socket.read_exact(&mut user_buf).await?;
    let username = String::from_utf8(user_buf).unwrap_or_default();

    // 3. 读取密码长度
    let mut plen_buf = [0u8; 1];
    socket.read_exact(&mut plen_buf).await?;
    let plen = plen_buf[0] as usize;

    // 4. 读取密码
    let mut pass_buf = vec![0u8; plen];
    socket.read_exact(&mut pass_buf).await?;
    let password = String::from_utf8(pass_buf).unwrap_or_default();

    debug!("[Auth] 尝试认证: {} / ***", username);

    // 5. 校验
    if username == user_config.username && password == user_config.password {
        socket.write_all(&[AUTH_VERSION, AUTH_SUCCESS]).await?;
        info!("用户 {} 认证成功", username);
        Ok(())
    } else {
        socket.write_all(&[AUTH_VERSION, AUTH_FAILURE]).await?;
        warn!("用户 {} 认证失败: 密码错误", username);
        Err("身份验证失败".into())
    }
}
