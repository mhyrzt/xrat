use serde_json::{Value, json};
use std::{
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Clone, Default)]
pub struct Events {
    entries: Arc<Mutex<Vec<Value>>>,
    file: Option<PathBuf>,
}
impl Events {
    pub fn file(path: PathBuf) -> Self {
        Self {
            file: Some(path),
            ..Self::default()
        }
    }
    pub fn record(&self, event: Value) {
        let mut entries = self.entries.lock().unwrap();
        if let Some(path) = &self.file {
            writeln!(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .unwrap(),
                "{event}"
            )
            .unwrap();
        }
        entries.push(event);
    }
    pub fn all(&self) -> Vec<Value> {
        if let Some(path) = &self.file {
            fs::read_to_string(path)
                .unwrap_or_default()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        } else {
            self.entries.lock().unwrap().clone()
        }
    }
}
pub fn exact(stream: &mut impl Read, size: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![0; size];
    stream.read_exact(&mut bytes)?;
    Ok(bytes)
}
fn number(bytes: &[u8]) -> u16 {
    u16::from_be_bytes(bytes.try_into().unwrap())
}
pub fn dns_reply(packet: &[u8], events: &Events, ipv6: &str) -> io::Result<Vec<u8>> {
    let mut offset = 12;
    let mut labels = Vec::new();
    loop {
        let length = *packet.get(offset).ok_or(io::ErrorKind::InvalidData)? as usize;
        offset += 1;
        if length == 0 {
            break;
        }
        labels.push(
            String::from_utf8_lossy(
                packet
                    .get(offset..offset + length)
                    .ok_or(io::ErrorKind::InvalidData)?,
            )
            .into_owned(),
        );
        offset += length;
    }
    let question = packet
        .get(12..offset + 4)
        .ok_or(io::ErrorKind::InvalidData)?;
    let kind = number(&packet[offset..offset + 2]);
    events.record(json!(["dns", labels.join("."), kind]));
    let address = match kind {
        1 => vec![203, 0, 113, 2],
        28 => ipv6
            .parse::<std::net::Ipv6Addr>()
            .unwrap()
            .octets()
            .to_vec(),
        _ => vec![],
    };
    let mut response = packet[..2].to_vec();
    for value in [0x8180u16, 1, u16::from(!address.is_empty()), 0, 0] {
        response.extend(value.to_be_bytes());
    }
    response.extend(question);
    if !address.is_empty() {
        response.extend([0xc0, 0x0c]);
        response.extend(kind.to_be_bytes());
        response.extend(1u16.to_be_bytes());
        response.extend(30u32.to_be_bytes());
        response.extend((address.len() as u16).to_be_bytes());
        response.extend(address);
    }
    Ok(response)
}
pub fn tcp_server(address: SocketAddr, handler: impl Fn(TcpStream) + Send + Sync + 'static) {
    let listener = TcpListener::bind(address).unwrap();
    let handler = Arc::new(handler);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let handler = handler.clone();
            thread::spawn(move || handler(stream.unwrap()));
        }
    });
}
pub fn start_dns(address: SocketAddr, events: &Events, ipv6: &'static str) {
    let socket = UdpSocket::bind(address).unwrap();
    let udp_events = events.clone();
    thread::spawn(move || {
        let mut bytes = [0; 4096];
        loop {
            let (length, peer) = socket.recv_from(&mut bytes).unwrap();
            if let Ok(answer) = dns_reply(&bytes[..length], &udp_events, ipv6) {
                socket.send_to(&answer, peer).unwrap();
            }
        }
    });
    let dns_events = events.clone();
    tcp_server(address, move |mut stream| {
        while let Ok(prefix) = exact(&mut stream, 2) {
            let result = (|| -> io::Result<()> {
                let packet = exact(&mut stream, number(&prefix) as usize)?;
                let response = dns_reply(&packet, &dns_events, ipv6)?;
                stream.write_all(&(response.len() as u16).to_be_bytes())?;
                stream.write_all(&response)
            })();
            if result.is_err() {
                break;
            }
        }
    });
}
pub fn start(events: Events, origins: &[&str], ipv6: &'static str, udp: bool) {
    start_dns("192.0.2.2:5353".parse().unwrap(), &events, ipv6);
    start_socks("192.0.2.2:1081".parse().unwrap(), &events);
    for host in origins {
        start_echo(host.parse().unwrap(), &events, udp);
    }
}

pub fn start_socks(address: SocketAddr, events: &Events) {
    let events = events.clone();
    tcp_server(address, move |stream| {
        let _ = socks(stream, &events);
    });
}

pub fn start_echo(address: IpAddr, events: &Events, udp: bool) {
    let echo_events = events.clone();
    tcp_server(SocketAddr::new(address, 8080), move |mut stream| {
        echo_events.record(json!([
            "origin-tcp",
            stream.peer_addr().unwrap().ip().to_string(),
            stream.local_addr().unwrap().ip().to_string()
        ]));
        if let Ok(bytes) = exact(&mut stream, 4) {
            let _ = stream.write_all(&bytes);
        }
    });
    if udp {
        let socket = UdpSocket::bind(SocketAddr::new(address, 8081)).unwrap();
        let echo_events = events.clone();
        thread::spawn(move || {
            let mut bytes = [0; 65536];
            loop {
                let (length, peer) = socket.recv_from(&mut bytes).unwrap();
                echo_events.record(json!([
                    "origin-udp",
                    peer.ip().to_string(),
                    address.to_string()
                ]));
                socket.send_to(&bytes[..length], peer).unwrap();
            }
        });
    }
}

fn read_host(stream: &mut impl Read, kind: u8) -> io::Result<String> {
    match kind {
        1 => Ok(
            std::net::Ipv4Addr::from(<[u8; 4]>::try_from(exact(stream, 4)?).unwrap()).to_string(),
        ),
        4 => Ok(
            std::net::Ipv6Addr::from(<[u8; 16]>::try_from(exact(stream, 16)?).unwrap()).to_string(),
        ),
        3 => {
            let size = exact(stream, 1)?[0] as usize;
            String::from_utf8(exact(stream, size)?).map_err(|_| io::ErrorKind::InvalidData.into())
        }
        _ => Err(io::ErrorKind::InvalidData.into()),
    }
}
fn socks(mut stream: TcpStream, events: &Events) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let greeting = exact(&mut stream, 2)?;
    if greeting[0] != 5 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    exact(&mut stream, greeting[1] as usize)?;
    stream.write_all(&[5, 0])?;
    let request = exact(&mut stream, 4)?;
    let host = read_host(&mut stream, request[3])?;
    let port = number(&exact(&mut stream, 2)?);
    if request[1] == 3 {
        return udp_associate(stream, events);
    }
    if request[1] != 1 {
        return Err(io::ErrorKind::InvalidData.into());
    }
    events.record(json!(["proxy", host, port]));
    let target = if host.ends_with(".test") {
        "203.0.113.2"
    } else {
        &host
    };
    let mut upstream = connect(target, port)?;
    stream.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0])?;
    let mut reader = upstream.try_clone()?;
    let mut writer = stream.try_clone()?;
    let handle = thread::spawn(move || {
        let _ = io::copy(&mut reader, &mut writer);
        let _ = writer.shutdown(Shutdown::Both);
    });
    let _ = io::copy(&mut stream, &mut upstream);
    let _ = upstream.shutdown(Shutdown::Both);
    let _ = handle.join();
    Ok(())
}
fn udp_associate(mut control: TcpStream, events: &Events) -> io::Result<()> {
    let relay = UdpSocket::bind("192.0.2.2:0")?;
    relay.set_read_timeout(Some(Duration::from_millis(100)))?;
    control.write_all(&[5, 0, 0, 1, 192, 0, 2, 2])?;
    control.write_all(&relay.local_addr()?.port().to_be_bytes())?;
    control.set_nonblocking(true)?;
    let mut buffer = [0; 65536];
    loop {
        if control.peek(&mut [0]).is_ok_and(|size| size == 0) {
            return Ok(());
        }
        let (length, peer) = match relay.recv_from(&mut buffer) {
            Ok(value) => value,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        let packet = &buffer[..length];
        if packet.get(..3) != Some(&[0, 0, 0]) {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut cursor = io::Cursor::new(&packet[4..]);
        let host = read_host(&mut cursor, packet[3])?;
        let port = number(&exact(&mut cursor, 2)?);
        let offset = 4 + cursor.position() as usize;
        events.record(json!(["proxy-udp", host, port]));
        let target: IpAddr = host.parse().map_err(|_| io::ErrorKind::InvalidData)?;
        let upstream = UdpSocket::bind(if target.is_ipv6() {
            "[::]:0"
        } else {
            "0.0.0.0:0"
        })?;
        upstream.set_read_timeout(Some(Duration::from_secs(10)))?;
        upstream.send_to(&packet[offset..], SocketAddr::new(target, port))?;
        let mut response = vec![0; 65536];
        let (size, _) = upstream.recv_from(&mut response)?;
        let mut answer = packet[..offset].to_vec();
        answer.extend(&response[..size]);
        relay.send_to(&answer, peer)?;
    }
}
pub fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
    let stream = TcpStream::connect_timeout(
        &SocketAddr::new(host.parse().map_err(|_| io::ErrorKind::InvalidInput)?, port),
        Duration::from_secs(10),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    Ok(stream)
}
pub fn lookup(
    name: &str,
    tcp: bool,
    kind: u16,
    host: &str,
    port: u16,
) -> io::Result<(u16, Option<IpAddr>)> {
    let mut packet = Vec::new();
    for value in [123u16, 0x0100, 1, 0, 0, 0] {
        packet.extend(value.to_be_bytes());
    }
    for label in name.split('.') {
        packet.push(label.len() as u8);
        packet.extend(label.as_bytes());
    }
    packet.push(0);
    packet.extend(kind.to_be_bytes());
    packet.extend(1u16.to_be_bytes());
    let response = if tcp {
        let mut stream = connect(host, port)?;
        stream.write_all(&(packet.len() as u16).to_be_bytes())?;
        stream.write_all(&packet)?;
        let size = number(&exact(&mut stream, 2)?) as usize;
        exact(&mut stream, size)?
    } else {
        let socket = UdpSocket::bind(if host.contains(':') {
            "[::]:0"
        } else {
            "0.0.0.0:0"
        })?;
        socket.set_read_timeout(Some(Duration::from_secs(10)))?;
        socket.connect(SocketAddr::new(host.parse().unwrap(), port))?;
        socket.send(&packet)?;
        let mut response = vec![0; 4096];
        let size = socket.recv(&mut response)?;
        response.truncate(size);
        response
    };
    assert_eq!(&response[..2], &packet[..2], "DNS transaction mismatch");
    let code = number(&response[2..4]) & 15;
    let count = number(&response[6..8]);
    let address = if count > 0 && code == 0 {
        Some(if kind == 28 {
            IpAddr::V6(std::net::Ipv6Addr::from(
                <[u8; 16]>::try_from(&response[response.len() - 16..]).unwrap(),
            ))
        } else {
            IpAddr::V4(std::net::Ipv4Addr::from(
                <[u8; 4]>::try_from(&response[response.len() - 4..]).unwrap(),
            ))
        })
    } else {
        None
    };
    Ok((code, address))
}
pub fn recover_fake(address: IpAddr) {
    let mut stream = connect("127.0.0.1", 1080).unwrap();
    stream.write_all(&[5, 1, 0]).unwrap();
    assert_eq!(exact(&mut stream, 2).unwrap(), [5, 0]);
    let mut request = vec![5, 1, 0];
    match address {
        IpAddr::V4(value) => {
            request.push(1);
            request.extend(value.octets());
        }
        IpAddr::V6(value) => {
            request.push(4);
            request.extend(value.octets());
        }
    }
    request.extend(8080u16.to_be_bytes());
    stream.write_all(&request).unwrap();
    assert_eq!(exact(&mut stream, 10).unwrap()[1], 0);
    stream.write_all(b"xrat").unwrap();
    assert_eq!(exact(&mut stream, 4).unwrap(), b"xrat");
}
pub fn send(host: &str, udp: bool) {
    if udp {
        let socket = UdpSocket::bind(if host.contains(':') {
            "[::]:0"
        } else {
            "0.0.0.0:0"
        })
        .unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        socket
            .connect(SocketAddr::new(host.parse().unwrap(), 8081))
            .unwrap();
        socket.send(b"xrat").unwrap();
        let mut response = [0; 4];
        assert_eq!(socket.recv(&mut response).unwrap(), 4);
        assert_eq!(&response, b"xrat");
    } else {
        let mut stream = connect(host, 8080).unwrap();
        stream.write_all(b"xrat").unwrap();
        assert_eq!(exact(&mut stream, 4).unwrap(), b"xrat");
    }
}
