use std::error::Error;
use std::net::SocketAddr;
use std::result;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, error, warn};

use crate::consts::*;
use crate::protocol::{Address, UDPAssociateHeader};

pub struct UDPRelay {
    socket: Arc<UdpSocket>,
    client_addr: Option<SocketAddr>,      // 记录 Client 的 UDP 地址
    expected_client_ip: std::net::IpAddr, // 握手时记录的 Client IP，用于安全校验
}

impl UDPRelay {
    pub async fn new(client_ip: std::net::IpAddr) -> Result<(Self, SocketAddr), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let listen_addr = socket.local_addr()?;

        Ok((
            UDPRelay {
                socket: Arc::new(socket),
                client_addr: None,
                expected_client_ip: client_ip,
            },
            listen_addr,
        ))
    }

    pub async fn run(mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = [0u8; MAX_UDP_SIZE as usize];
        loop {
            let res = timeout(
                Duration::from_secs(UDP_TIMEOUT as u64),
                self.socket.recv_from(&mut buf),
            )
            .await;

            let (len, src_addr) = match res {
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    error!("udp read error:{}", e);
                    continue;
                }
                Err(_) => {
                    debug!("udp timeout, closed");
                    return Ok(());
                }
            };

            if self.is_from_client(&src_addr) {
                // 来自客户端 -> 发往目标
                if let Err(e) = self.handle_outbound(&buf[..len], src_addr).await {
                    debug!("handle outbound error: {}", e);
                }
            } else {
                // 来自目标 -> 发回客户端
                if let Err(e) = self.handle_inbound(&buf[..len], src_addr).await {
                    debug!("handle inbound error: {}", e);
                }
            }
        }
    }

    fn is_from_client(&self, addr: &SocketAddr) -> bool {
        // 1. IP 必须匹配握手时的 IP
        if addr.ip() != self.expected_client_ip {
            return false;
        }

        // 2. 如果我们已经记录了 Client 的完整地址 (IP+Port)，则直接匹配
        if let Some(client) = self.client_addr {
            return client == *addr;
        }

        // 3. 如果是第一次收到该 IP 的包，我们默认它就是 Client，并记录 Port
        // (SOCKS5 允许 Client 使用动态端口发送 UDP)
        true
    }

    async fn handle_outbound(
        &mut self,
        packet: &[u8],
        src_addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        if self.client_addr.is_none() {
            debug!("lock udp client:{}", src_addr);
            self.client_addr = Some(src_addr);
        }

        let (header, header_len) = UDPAssociateHeader::parse(packet)?;

        if header.frag != 0 {
            warn!("unsupport UDP frag...");
            return Ok(());
        }

        let payload = &packet[header_len..];

        let target_addr = format!("{:?}:{:?}", header.address, header.port);
        self.socket.send_to(payload, target_addr).await?;
        Ok(())
    }

    async fn handle_inbound(
        &self,
        payload: &[u8],
        src_addr: SocketAddr,
    ) -> Result<(), Box<dyn Error>> {
        let client_addr = match self.client_addr {
            Some(addr) => addr,
            None => return Err("unkonw client addr".into()),
        };

        let address = match src_addr.ip() {
            std::net::IpAddr::V4(ip) => Address::IpV4(ip),
            std::net::IpAddr::V6(ip) => Address::IpV6(ip),
        };

        let header = UDPAssociateHeader {
            frag: 0,
            address,
            port: src_addr.port(),
        };

        // 2. 序列化 Header + Payload
        let mut send_buf = Vec::with_capacity(20 + payload.len());
        header.write(&mut send_buf);
        send_buf.extend_from_slice(payload);

        // 3. 发回 Client
        self.socket.send_to(&send_buf, client_addr).await?;

        Ok(())
    }
}
