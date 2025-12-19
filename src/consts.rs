// +----+----------+----------+
// |VER | NMETHODS | METHODS  |
// +----+----------+----------+
// | 1  |    1     | 1 to 255 |
// +----+----------+----------+

// +----+------+----------+------+----------+
// |VER | ULEN |  UNAME   | PLEN |  PASSWD  |
// +----+------+----------+------+----------+
// | 1  |  1   | 1 to 255 |  1   | 1 to 255 |
// +----+------+----------+------+----------+

// +----+-----+-------+------+----------+----------+
// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

pub const SOCKS_VERSION: u8 = 0x05;

// auth methods
pub const METHOD_NO_AUTH: u8 = 0x00;
pub const METHOD_GASSAPI: u8 = 0x01;
pub const METHOD_PASSWORD: u8 = 0x02;
pub const METHOD_NO_ACCEPTABLE: u8 = 0xFF;

//
pub const AUTH_VERSION: u8 = 0x01;
pub const AUTH_SUCCESS: u8 = 0x00;
pub const AUTH_FAILURE: u8 = 0x01;

// command CMD
pub const CMD_CONNECT: u8 = 0x01;

// address type ATYP
pub const ATYP_IPV4: u8 = 0x01;
pub const ATYP_DOMAIN: u8 = 0x03;
pub const ATYP_IPV6: u8 = 0x04;

// SOCKS5 响应码 (REP)
pub const REP_SUCCESS: u8 = 0x00;
pub const REP_GENERAL_FAILURE: u8 = 0x01;
pub const REP_CONNECTION_NOT_ALLOWED: u8 = 0x02;
pub const REP_NETWORK_UNREACHABLE: u8 = 0x03;
pub const REP_HOST_UNREACHABLE: u8 = 0x04;
pub const REP_CONNECTION_REFUSED: u8 = 0x05;
pub const REP_TTL_EXPIRED: u8 = 0x06;
pub const REP_COMMAND_NOT_SUPPORTED: u8 = 0x07;
pub const REP_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;

// UDP
pub const CMD_UDP_ASSOCIATE: u8 = 0x03;
pub const RSV: u8 = 0x00;
pub const FRAG: u8 = 0x00; // SOCKS5 分片字段，通常不实现（填0）

pub const MAX_UDP_SIZE: u64 = 65535;
pub const UDP_TIMEOUT: usize = 300;
