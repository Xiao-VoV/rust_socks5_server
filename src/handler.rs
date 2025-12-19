use std::error::Error;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

// 引入我们封装好的模块
use crate::auth::{self, UserConfig};
use crate::consts::*;
use crate::protocol::SocksRequest;
use crate::udp::UDPRelay;

pub async fn process(mut socket: TcpStream, config: &UserConfig) -> Result<(), Box<dyn Error>> {
    // ==========================================
    // 阶段 1: 协商 (Handshake)
    // ==========================================

    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    if buf[0] != SOCKS_VERSION {
        return Err("仅支持 SOCKS5 协议".into());
    }

    // 读取 NMETHODS
    let mut buf = [0u8; 1];
    socket.read_exact(&mut buf).await?;
    let nmethods = buf[0] as usize;

    let mut methods = vec![0u8; nmethods];
    socket.read_exact(&mut methods).await?;

    let mut should_auth = false;

    if let Some(_user_config) = &config.user {
        if methods.contains(&METHOD_PASSWORD) {
            should_auth = true;
            socket.write_all(&[SOCKS_VERSION, METHOD_PASSWORD]).await?;
        } else {
            socket
                .write_all(&[SOCKS_VERSION, METHOD_NO_ACCEPTABLE])
                .await?;
            return Err("client don't support auth".into());
        }
    } else {
        socket.write_all(&[SOCKS_VERSION, METHOD_NO_AUTH]).await?;
    }

    if should_auth {
        auth::perform_password_auth(&mut socket, config.user.as_ref().unwrap()).await?;
    }
    // ==========================================
    // 阶段 2: 请求 (Request)
    // ==========================================

    let request = SocksRequest::read_from(&mut socket).await?;

    // 根据命令分发到不同的处理函数
    match request.cmd {
        CMD_CONNECT => {
            handle_tcp_connect(socket, request, config).await?;
        }
        CMD_UDP_ASSOCIATE => {
            handle_udp_associate(socket, request).await?;
        }
        _ => {
            warn!("不支持的命令: {}", request.cmd);
            let reply = [
                SOCKS_VERSION,
                REP_COMMAND_NOT_SUPPORTED,
                0x00,
                ATYP_IPV4,
                0,
                0,
                0,
                0,
                0,
                0,
            ];
            socket.write_all(&reply).await?;
            return Err("Unsupported Command".into());
        }
    }

    Ok(())
}

/// 处理 TCP CONNECT 命令
async fn handle_tcp_connect(
    mut socket: TcpStream,
    request: SocksRequest,
    config: &UserConfig,
) -> Result<(), Box<dyn Error>> {
    let target = request.to_string();
    info!("TCP Connect to: {}", target);

    // ==========================================
    // 阶段 3: TCP 转发
    // ==========================================
    let connect_timeout = Duration::from_secs(config.timeout as u64);
    let server_socket_result = timeout(connect_timeout, TcpStream::connect(&target)).await;

    let mut server_socket = match server_socket_result {
        Err(_) => {
            warn!("连接目标超时 ({}s): {}", config.timeout, target);
            let reply = [
                SOCKS_VERSION,
                REP_TTL_EXPIRED,
                0x00,
                ATYP_IPV4,
                0,
                0,
                0,
                0,
                0,
                0,
            ];
            let _ = socket.write_all(&reply).await;
            return Err("连接目标超时".into());
        }
        Ok(connect_result) => match connect_result {
            Ok(s) => s,
            Err(e) => {
                let rep = match e.kind() {
                    std::io::ErrorKind::ConnectionRefused => REP_CONNECTION_REFUSED,
                    std::io::ErrorKind::TimedOut => REP_NETWORK_UNREACHABLE,
                    std::io::ErrorKind::PermissionDenied => REP_CONNECTION_NOT_ALLOWED,
                    _ => REP_HOST_UNREACHABLE,
                };
                error!("目标主机连接失败：{}({})", target, e);
                let reply = [SOCKS_VERSION, rep, 0x00, ATYP_IPV4, 0, 0, 0, 0, 0, 0];
                let _ = socket.write_all(&reply).await;
                return Err(e.into());
            }
        },
    };

    // 告诉客户端连接成功
    let reply = [
        SOCKS_VERSION,
        REP_SUCCESS,
        0x00,
        ATYP_IPV4,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    socket.write_all(&reply).await?;

    transfer(&mut socket, &mut server_socket).await?;

    Ok(())
}

/// 处理 UDP ASSOCIATE 命令
async fn handle_udp_associate(
    mut socket: TcpStream,
    _request: SocksRequest, // UDP Associate 请求中的 IP/Port 通常被忽略，或者是客户端希望发送 UDP 的源地址
) -> Result<(), Box<dyn Error>> {
    let client_ip = socket.peer_addr()?.ip();
    info!("UDP Associate request from: {}", client_ip);

    // 1. 初始化 UDP Relay
    // 这会绑定一个随机 UDP 端口
    let (relay, listen_addr) = UDPRelay::new(client_ip).await?;
    let udp_port = listen_addr.port();

    info!("UDP Relay started at port: {}", udp_port);

    // 2. 告诉客户端 UDP 监听端口
    // 这里回复 IP 0.0.0.0 (全0)，告诉客户端使用它连接 TCP 时使用的那个服务器 IP
    let reply = [
        SOCKS_VERSION,
        REP_SUCCESS,
        0x00,
        ATYP_IPV4,
        0,
        0,
        0,
        0, // IP
        (udp_port >> 8) as u8,
        (udp_port & 0xff) as u8, // Port
    ];
    socket.write_all(&reply).await?;

    // 3. 并发运行：UDP 转发循环 & TCP 保活监控
    // SOCKS5 规定：当 TCP 断开时，UDP 关联也必须停止
    let mut keepalive_buf = [0u8; 1];
    tokio::select! {
        res = relay.run() => {
            if let Err(e) = res {
                error!("UDP Relay error: {}", e);
            }
        }
        // 监控 TCP socket，读取 1 字节
        // 如果返回 0 (EOF) 或错误，说明 TCP 断开了
        res = socket.read(&mut keepalive_buf) => {
            match res {
                Ok(0) => debug!("Client closed TCP connection, stopping UDP"),
                Ok(_) => warn!("Unexpected data on TCP control channel"),
                Err(e) => warn!("TCP connection error: {}", e),
            }
        }
    }

    Ok(())
}

async fn transfer(client: &mut TcpStream, server: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    #[cfg(target_os = "linux")]
    {
        use tokio_splice::zero_copy_bidirectional;

        // splice 需要文件描述符，tokio 的 TcpStream 实现了 AsRawFd
        match zero_copy_bidirectional(client, server).await {
            Ok((up, down)) => {
                debug!("Splice 传输完成: 上行 {}b, 下行 {}b", up, down);
                Ok(())
            }
            Err(e) => {
                error!("Splice 传输错误: {}", e);
                Err(e.into())
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        // 非 Linux (macOS/Windows) 使用普通的用户态拷贝
        match tokio::io::copy_bidirectional(client, server).await {
            Ok((up, down)) => {
                debug!("Copy 传输完成: 上行 {}b, 下行 {}b", up, down);
                Ok(())
            }
            Err(e) => {
                // copy_bidirectional 有时在断开时会报 ConnectionReset，这其实不算严重错误
                debug!("Copy 传输中断: {}", e);
                Ok(())
            }
        }
    }
}
