use std::io;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::time::Duration;

use hickory_proto::op::{Message, MessageType, OpCode, Query, ResponseCode};
use hickory_proto::rr::{Name, RData, RecordType};

pub fn resolve_udp(
    host: &str,
    server: SocketAddr,
    ipv4: bool,
    ipv6: bool,
    timeout: Duration,
) -> io::Result<Vec<IpAddr>> {
    let mut name = Name::from_ascii(host).map_err(invalid)?;
    name.set_fqdn(true);
    let socket = UdpSocket::bind(if server.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    })?;
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))?;
    socket.connect(server)?;
    let mut addresses = Vec::new();
    let mut last_error = None;
    for kind in [RecordType::A, RecordType::AAAA] {
        if (kind == RecordType::A && !ipv4) || (kind == RecordType::AAAA && !ipv6) {
            continue;
        }
        match query(&socket, &name, kind) {
            Ok(found) => addresses.extend(found),
            Err(error) => last_error = Some(error),
        }
    }
    if addresses.is_empty() {
        return Err(last_error.unwrap_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "bootstrap DNS returned no usable address",
            )
        }));
    }
    Ok(addresses)
}

fn query(socket: &UdpSocket, name: &Name, kind: RecordType) -> io::Result<Vec<IpAddr>> {
    let random = uuid::Uuid::new_v4();
    let id = u16::from_be_bytes([random.as_bytes()[0], random.as_bytes()[1]]);
    let question = Query::query(name.clone(), kind);
    let mut request = Message::new(id, MessageType::Query, OpCode::Query);
    request.metadata.recursion_desired = true;
    request.add_query(question.clone());
    socket.send(&request.to_vec().map_err(invalid)?)?;
    let mut bytes = [0; 4096];
    let count = socket.recv(&mut bytes)?;
    let reply = Message::from_vec(&bytes[..count]).map_err(invalid)?;
    if reply.metadata.id != id
        || reply.metadata.message_type != MessageType::Response
        || reply.metadata.op_code != OpCode::Query
        || reply.queries != [question]
        || reply.metadata.truncation
        || reply.metadata.response_code != ResponseCode::NoError
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bootstrap DNS response is invalid, truncated or unsuccessful",
        ));
    }
    let mut names = vec![name.clone()];
    for _ in 0..reply.answers.len() {
        for answer in &reply.answers {
            if names.contains(&answer.name)
                && let RData::CNAME(alias) = &answer.data
                && !names.contains(&alias.0)
            {
                names.push(alias.0.clone());
            }
        }
    }
    Ok(reply
        .answers
        .iter()
        .filter(|answer| names.contains(&answer.name))
        .filter_map(|answer| match &answer.data {
            RData::A(address) if kind == RecordType::A => Some(IpAddr::V4(address.0)),
            RData::AAAA(address) if kind == RecordType::AAAA => Some(IpAddr::V6(address.0)),
            _ => None,
        })
        .collect())
}

fn invalid(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
