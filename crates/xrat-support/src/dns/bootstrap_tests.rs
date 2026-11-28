use super::*;
use hickory_proto::rr::{
    Record,
    rdata::{A, AAAA, CNAME},
};

fn resolver(
    mut respond: impl FnMut(Message) -> Message + Send + 'static,
    count: usize,
) -> (SocketAddr, std::thread::JoinHandle<()>) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let address = socket.local_addr().unwrap();
    let thread = std::thread::spawn(move || {
        for _ in 0..count {
            let mut bytes = [0; 4096];
            let (size, peer) = socket.recv_from(&mut bytes).unwrap();
            let request = Message::from_vec(&bytes[..size]).unwrap();
            let reply = respond(request);
            socket.send_to(&reply.to_vec().unwrap(), peer).unwrap();
        }
    });
    (address, thread)
}

fn response(request: &Message) -> Message {
    let mut reply = Message::response(request.metadata.id, OpCode::Query);
    reply.queries.clone_from(&request.queries);
    reply
}

#[test]
fn follows_cname_answers_and_filters_unrelated_addresses_for_both_families() {
    let (server, thread) = resolver(
        |request| {
            let mut reply = response(&request);
            let alias = Name::from_ascii("alias.test").unwrap();
            let record = match request.queries[0].query_type() {
                RecordType::A => RData::A(A("192.0.2.3".parse().unwrap())),
                RecordType::AAAA => RData::AAAA(AAAA("2001:db8::3".parse().unwrap())),
                _ => unreachable!(),
            };
            reply.add_answer(Record::from_rdata(alias.clone(), 60, record.clone()));
            reply.add_answer(Record::from_rdata(
                Name::from_ascii("unrelated.test").unwrap(),
                60,
                record,
            ));
            reply.add_answer(Record::from_rdata(
                request.queries[0].name().clone(),
                60,
                RData::CNAME(CNAME(alias)),
            ));
            reply
        },
        2,
    );
    let addresses = resolve_udp("origin.test", server, true, true, Duration::from_secs(1)).unwrap();
    assert_eq!(
        addresses,
        vec![
            "192.0.2.3".parse::<IpAddr>().unwrap(),
            "2001:db8::3".parse().unwrap()
        ]
    );
    thread.join().unwrap();
}

#[test]
fn rejects_wrong_transaction_question_truncation_and_dns_errors() {
    for change in 0..4 {
        let (server, thread) = resolver(
            move |request| {
                let mut reply = response(&request);
                match change {
                    0 => {
                        reply.metadata.id = request.metadata.id.wrapping_add(1);
                    }
                    1 => {
                        reply.queries.clear();
                        reply.add_query(Query::query(
                            Name::from_ascii("other.test").unwrap(),
                            RecordType::A,
                        ));
                    }
                    2 => {
                        reply.metadata.truncation = true;
                    }
                    _ => {
                        reply.metadata.response_code = ResponseCode::Refused;
                    }
                }
                reply
            },
            1,
        );
        assert_eq!(
            resolve_udp("origin.test", server, true, false, Duration::from_secs(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        thread.join().unwrap();
    }
}

#[test]
fn empty_and_disabled_family_results_are_errors() {
    let (server, thread) = resolver(|request| response(&request), 1);
    assert_eq!(
        resolve_udp("origin.test", server, false, true, Duration::from_secs(1))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    thread.join().unwrap();
    assert_eq!(
        resolve_udp("origin.test", server, false, false, Duration::from_secs(1))
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
}

#[test]
fn bounds_waiting_for_a_silent_resolver() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let started = std::time::Instant::now();
    let error = resolve_udp(
        "origin.test",
        socket.local_addr().unwrap(),
        true,
        false,
        Duration::from_millis(40),
    )
    .unwrap_err();
    assert!(matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    assert!(started.elapsed() < Duration::from_secs(1));
}
