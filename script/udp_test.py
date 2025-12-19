import socket
import struct
import time

# 配置你的代理地址
PROXY_IP = '127.0.0.1'
PROXY_PORT = 8080 # 对应你 Rust 代码里的端口
# 配置你的代理认证 (如果没有就留空或修改脚本)
USERNAME = 'admin'
PASSWORD = '123'
ENABLE_AUTH = True # 如果你的 Rust 代码开启了 force auth，设为 True

def test_udp():
    # 1. 建立 TCP 控制连接
    tcp_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    tcp_sock.connect((PROXY_IP, PROXY_PORT))
    print(f"[TCP] Connected to proxy {PROXY_IP}:{PROXY_PORT}")

    # 2. 协商阶段 (Handshake)
    tcp_sock.sendall(b'\x05\x02\x00\x02') # Ver 5, 2 Methods, NoAuth(00) & UserPass(02)
    ver, method = struct.unpack('BB', tcp_sock.recv(2))
    
    if method == 0x02 and ENABLE_AUTH:
        print("[TCP] Doing Username/Password Auth...")
        # 发送鉴权包: Ver 1 + Ulen + User + Plen + Pass
        auth_payload = b'\x01' + bytes([len(USERNAME)]) + USERNAME.encode() + \
                       bytes([len(PASSWORD)]) + PASSWORD.encode()
        tcp_sock.sendall(auth_payload)
        ver, status = struct.unpack('BB', tcp_sock.recv(2))
        if status != 0:
            print("Authentication failed!")
            return
    elif method == 0xFF:
        print("No acceptable methods (Server requires auth but client sent none?)")
        return
    
    print("[TCP] Handshake & Auth success")

    # 3. 请求 UDP Associate
    # Ver 5, Cmd 3 (UDP), Rsv 0, Atyp 1 (IPv4), 0.0.0.0:0
    # 注意：客户端告诉代理 "我想发 UDP"，后面的 IP:Port 通常填 0，由代理决定分配什么
    req = b'\x05\x03\x00\x01\x00\x00\x00\x00\x00\x00'
    tcp_sock.sendall(req)

    # 4. 读取响应，获取 Relay 端口
    resp = tcp_sock.recv(10)
    ver, rep, rsv, atyp = struct.unpack('BBBB', resp[:4])
    
    if rep != 0x00:
        print(f"[TCP] UDP Associate request failed with REP: {rep}")
        return

    # 解析代理分配的 UDP IP 和 端口
    relay_ip_bytes = resp[4:8]
    relay_port_bytes = resp[8:10]
    relay_ip = socket.inet_ntoa(relay_ip_bytes)
    relay_port = struct.unpack('!H', relay_port_bytes)[0]
    
    # 修正：如果代理返回 0.0.0.0，意味着我们要发给 Proxy 的 IP
    if relay_ip == '0.0.0.0':
        relay_ip = PROXY_IP

    print(f"[UDP] Proxy assigned relay address: {relay_ip}:{relay_port}")

    # 5. 准备 UDP 数据发送
    udp_sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    udp_sock.settimeout(5)

    # 构造 DNS 查询包 (查询 google.com A 记录)
    # 这是一个标准的 DNS query payload
    dns_query = b'\xAA\xAA\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00' \
                b'\x06google\x03com\x00\x00\x01\x00\x01'
    
    # 构造 SOCKS5 UDP 头
    # RSV(2) | FRAG(1) | ATYP(1) | DST.ADDR(4) | DST.PORT(2)
    # 我们要把包发给 8.8.8.8:53
    dest_ip = socket.inet_aton('8.8.8.8')
    dest_port = struct.pack('!H', 53)
    header = b'\x00\x00\x00\x01' + dest_ip + dest_port
    
    packet = header + dns_query

    # 6. 发送 UDP 包到代理的 Relay 端口
    print(f"[UDP] Sending DNS query via proxy to 8.8.8.8:53...")
    udp_sock.sendto(packet, (relay_ip, relay_port))

    # 7. 接收响应
    try:
        data, _ = udp_sock.recvfrom(4096)
        # 解析 SOCKS5 UDP 头
        # 头部的长度取决于 ATYP。如果是 IPv4(1)，头长 10 字节
        if data[3] == 1:
            header_len = 10
        else:
            header_len = 0 # 简化处理，测试只用了 IPv4

        real_payload = data[header_len:]
        
        print(f"[UDP] Received {len(data)} bytes from proxy")
        print(f"[UDP] Raw Response (Hex): {real_payload[:20].hex()}...")
        
        if b'google' in real_payload or b'com' in real_payload:
             print("\n✅ 测试成功！成功通过 UDP 代理收到了 DNS 响应。")
        else:
             print("\n❓ 收到数据，但看起来不像 DNS 响应。")

    except socket.timeout:
        print("\n❌ 测试失败：UDP 接收超时 (代理没有转发回来，或者防火墙拦截)")
    finally:
        tcp_sock.close()
        udp_sock.close()

if __name__ == '__main__':
    test_udp()