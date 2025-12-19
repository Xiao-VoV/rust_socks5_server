use std::error::Error;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Add;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::consts::*;

#[derive(Debug)]
pub enum Address {
    IpV4(Ipv4Addr),
    Domain(String),
    IpV6(Ipv6Addr),
}

#[derive(Debug)]
pub struct SocksRequest {
    pub cmd: u8,
    pub address: Address,
    pub port: u16,
}

impl fmt::Display for SocksRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.address {
            Address::IpV4(ip) => write!(f, "{}:{}", ip, self.port),
            Address::Domain(domain) => write!(f, "{}:{}", domain, self.port),
            Address::IpV6(ip) => write!(f, "[{}],{}", ip, self.port),
        }
    }
}

impl SocksRequest {
    pub async fn read_from(socket: &mut TcpStream) -> Result<Self, Box<dyn Error>> {
        let mut head = [0u8; 4];
        socket.read_exact(&mut head).await?;

        let ver = head[0];
        let cmd = head[1];
        let atyp = head[3];

        if ver != SOCKS_VERSION {
            return Err(format!("unsupport socks version: 0x{:02x}", ver).into());
        }

        let address = match atyp {
            ATYP_IPV4 => {
                let mut buf = [0u8; 4];
                socket.read_exact(&mut buf).await?;
                Address::IpV4(Ipv4Addr::from(buf))
            }
            ATYP_DOMAIN => {
                // 先读 1 字节长度
                let mut len_buf = [0u8; 1];
                socket.read_exact(&mut len_buf).await?;
                let len = len_buf[0] as usize;

                // 再读 N 字节域名
                let mut buf = vec![0u8; len];
                socket.read_exact(&mut buf).await?;

                // 转换成 String
                let domain = String::from_utf8(buf).map_err(|_| "wrong domain")?;
                Address::Domain(domain)
            }
            ATYP_IPV6 => {
                let mut buf = [0u8; 16];
                socket.read_exact(&mut buf).await?;
                Address::IpV6(Ipv6Addr::from(buf))
            }
            _ => return Err(format!("unknow address type: 0x{:02x}", atyp).into()),
        };

        let mut port_buf = [0u8; 2];
        socket.read_exact(&mut port_buf).await?;
        let port = u16::from_be_bytes(port_buf);

        Ok(SocksRequest { cmd, address, port })
    }
}

/// SOCKS5 UDP 数据报文头
/// +----+------+------+----------+----------+----------+
/// |RSV | FRAG | ATYP | DST.ADDR | DST.PORT |   DATA   |
/// +----+------+------+----------+----------+----------+
/// | 2  |  1   |  1   | Variable |    2     | Variable |
/// +----+------+------+----------+----------+----------+

#[derive(Debug)]
pub struct UDPAssociateHeader {
    pub frag: u8,
    pub address: Address,
    pub port: u16,
}

impl UDPAssociateHeader {
    pub fn parse(buf: &[u8]) -> Result<(Self, usize), Box<dyn Error>> {
        if buf.len() < 4 {
            return Err("UDP packet too short".into());
        }

        if buf[0] != 0x00 || buf[1] != 0x00 {
            return Err("Invalid reserved filds in UDP Header".into());
        }

        let frag = buf[2];
        let atyp = buf[3];

        let (address, port, consumed) = match atyp {
            ATYP_IPV4 => {
                if buf.len() < 10 {
                    return Err("IPv4 packet too short".into());
                }
                let ip = Ipv4Addr::new(buf[4], buf[5], buf[6], buf[7]);
                let port = u16::from_be_bytes([buf[8], buf[9]]);
                (Address::IpV4(ip), port, 10)
            }
            ATYP_IPV6 => {
                if buf.len() < 22 {
                    return Err("IPv6 packet too short".into());
                }
                let bytes: [u8; 16] = buf[4..20].try_into()?;
                let ip = Ipv6Addr::from(bytes);
                let port = u16::from_be_bytes([buf[20], buf[21]]);
                (Address::IpV6(ip), port, 22)
            }
            ATYP_DOMAIN => {
                let len = buf[4] as usize;
                if buf.len() < 5 + len + 2 {
                    return Err("Domain packet too short".into());
                }
                let domain_bytes = &buf[5..5 + len];
                let domain = String::from_utf8(domain_bytes.to_vec())?;
                let port_bytes = &buf[5 + len..5 + len + 2];
                let port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]);
                (Address::Domain(domain), port, 5 + len + 2)
            }
            _ => return Err(format!("Unknown ATYP: {}", atyp).into()),
        };

        Ok((
            UDPAssociateHeader {
                frag,
                address,
                port,
            },
            consumed,
        ))
    }
    pub fn write(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&[0x00, 0x00, self.frag]);

        match &self.address {
            Address::IpV4(ip) => {
                buf.push(ATYP_IPV4);
                buf.extend_from_slice(&ip.octets());
            }
            Address::Domain(domain) => {
                buf.push(ATYP_DOMAIN);
                buf.push(domain.len() as u8);
                buf.extend_from_slice(domain.as_bytes());
            }
            Address::IpV6(ip) => {
                buf.push(ATYP_IPV6);
                buf.extend_from_slice(&ip.octets());
            }
        }
        buf.extend_from_slice(&self.port.to_be_bytes());
    }
}
