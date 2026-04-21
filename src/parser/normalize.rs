use crate::model::Node;

pub fn normalize(node: &mut Node) {
    if node.network.is_empty() {
        node.network = "tcp".to_string();
    }

    if node.network == "ws" {
        if node.host.is_none() {
            node.host = node.sni.clone();
        }
        if node.path.is_none() {
            node.path = Some("/".to_string());
        }
    }

    if node.network == "grpc" && node.path.is_none() {
        node.path = Some("/".to_string());
    }

    if matches!(node.tls.as_deref(), Some("")) {
        node.tls = None;
    }
}
