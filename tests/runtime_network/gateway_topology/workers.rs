use super::*;

pub fn echo_worker() {
    process::assert_isolated();
    let events =
        fixtures::Events::file(std::env::var_os("XRAT_GATEWAY_ECHO_EVENTS").unwrap().into());
    if std::env::var_os("XRAT_GATEWAY_SOCKS_SERVER").is_some() {
        fixtures::start(events.clone(), &[CAPTURE_ECHO], "::1", true);
        fixtures::start_socks("192.0.2.2:1082".parse().unwrap(), &events);
    } else {
        fixtures::start_echo(CAPTURE_ECHO.parse().unwrap(), &events, true);
    }
    fixtures::start_dns(
        std::net::SocketAddr::new(CAPTURE_ECHO.parse().unwrap(), 53),
        &events,
        "::1",
    );
    std::fs::write(std::env::var_os("XRAT_GATEWAY_ECHO_READY").unwrap(), "").unwrap();
    loop {
        std::thread::park();
    }
}

pub fn client_worker() {
    process::assert_isolated();
    fixtures::send(CAPTURE_ECHO, false);
    fixtures::send(CAPTURE_ECHO, true);
    for (name, tcp) in [("gateway-udp.test", false), ("gateway-tcp.test", true)] {
        assert_eq!(
            fixtures::lookup(name, tcp, 1, CAPTURE_ECHO, 53).unwrap(),
            (0, Some("203.0.113.2".parse().unwrap()))
        );
    }
}

pub fn closed_client_worker() {
    use std::{
        io,
        net::{SocketAddr, TcpStream, UdpSocket},
        time::Duration,
    };

    process::assert_isolated();
    let destination = CAPTURE_ECHO.parse().unwrap();
    let timeout = Duration::from_millis(600);
    assert!(
        TcpStream::connect_timeout(&SocketAddr::new(destination, 8080), timeout).is_err(),
        "client reached the TCP origin while the gateway should be closed"
    );
    let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
    udp.connect(SocketAddr::new(destination, 8081)).unwrap();
    udp.set_read_timeout(Some(timeout)).unwrap();
    if udp.send(b"xrat").is_ok() {
        let error = udp
            .recv(&mut [0; 8])
            .expect_err("UDP origin bypassed the closed gateway");
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::ConnectionRefused
        ));
    }
}
