use super::*;

pub(super) struct Peer {
    worker: ChildGuard,
}

impl Peer {
    pub(super) fn new(port: &str, address: &str, bridge: Option<&str>) -> Self {
        let worker = ChildGuard(
            Command::new("unshare")
                .args(["--net", "sleep", "infinity"])
                .spawn()
                .unwrap(),
        );
        let peer = Self { worker };
        let pid = peer.worker.0.id().to_string();
        let gateway_namespace = std::fs::read_link("/proc/self/ns/net").unwrap();
        process::wait_for(
            || {
                std::fs::read_link(format!("/proc/{pid}/ns/net"))
                    .is_ok_and(|namespace| namespace != gateway_namespace)
            },
            5,
            "peer network namespace was not created",
        );
        process::ip(&[
            "link", "add", port, "type", "veth", "peer", "name", "lan", "netns", &pid,
        ]);
        if let Some(bridge) = bridge {
            process::ip(&["link", "set", port, "master", bridge]);
        }
        process::ip(&["link", "set", port, "up"]);
        peer.ip(&["link", "set", "lo", "up"]);
        peer.ip(&["address", "add", address, "dev", "lan"]);
        peer.ip(&["link", "set", "lan", "up"]);
        peer
    }

    pub(super) fn command(&self, program: &str) -> Command {
        let mut command = Command::new("nsenter");
        command.args([
            "--target",
            &self.worker.0.id().to_string(),
            "--net",
            program,
        ]);
        command
    }

    pub(super) fn ip(&self, arguments: &[&str]) {
        process::checked(self.command("ip").args(arguments));
    }
}
