use std::fmt;

use crate::model::Protocol;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeDedupKey {
    pub protocol: Protocol,
    pub address: String,
    pub port: u16,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
}

impl fmt::Display for NodeDedupKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("v1")?;
        write_required(f, "protocol", self.protocol.as_str())?;
        write_required(f, "address", &self.address)?;
        write_required(f, "port", &self.port.to_string())?;
        write_optional(f, "username", self.username.as_deref())?;
        write_optional(f, "uuid", self.uuid.as_deref())?;
        write_optional(f, "password", self.password.as_deref())?;
        write_optional(f, "method", self.method.as_deref())?;
        write_required(f, "network", &self.network)?;
        write_optional(f, "tls", self.tls.as_deref())?;
        write_optional(f, "sni", self.sni.as_deref())?;
        write_optional(f, "host", self.host.as_deref())?;
        write_optional(f, "path", self.path.as_deref())?;
        Ok(())
    }
}

fn write_required(f: &mut fmt::Formatter<'_>, name: &str, value: &str) -> fmt::Result {
    write!(f, "|{}={}:{}", name, value.chars().count(), value)
}

fn write_optional(f: &mut fmt::Formatter<'_>, name: &str, value: Option<&str>) -> fmt::Result {
    match value {
        Some(value) => write_required(f, name, value),
        None => write!(f, "|{}=-", name),
    }
}

#[cfg(test)]
mod tests {
    use super::NodeDedupKey;
    use crate::model::Protocol;

    #[test]
    fn formats_as_versioned_length_prefixed_key() {
        let key = NodeDedupKey {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid|123".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("cdn.example.com".to_string()),
            host: Some("cdn.example.com".to_string()),
            path: Some("/ray".to_string()),
        };

        assert_eq!(
            key.to_string(),
            "v1|protocol=5:vless|address=11:example.com|port=3:443|username=-|uuid=8:uuid|123|password=-|method=-|network=2:ws|tls=3:tls|sni=15:cdn.example.com|host=15:cdn.example.com|path=4:/ray"
        );
    }

    #[test]
    fn distinguishes_none_from_empty_string() {
        let none_key = NodeDedupKey {
            protocol: Protocol::Ss,
            address: "example.com".to_string(),
            port: 8388,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };
        let empty_key = NodeDedupKey {
            password: Some(String::new()),
            ..none_key.clone()
        };

        assert_ne!(none_key.to_string(), empty_key.to_string());
        assert!(empty_key.to_string().contains("|password=0:"));
    }

    #[test]
    fn handles_unicode_multibyte_characters() {
        let key = NodeDedupKey {
            protocol: Protocol::Vless,
            address: "例え.jp".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: Some("/路径".to_string()),
        };

        let output = key.to_string();
        // "例え.jp" has 5 chars (2 Japanese + 1 dot + 2 ASCII)
        // "/路径" has 3 chars (1 slash + 2 Chinese)
        assert!(
            output.contains("address=5:例え.jp"),
            "output was: {}",
            output
        );
        assert!(output.contains("path=3:/路径"), "output was: {}", output);
    }

    #[test]
    fn handles_special_characters_in_fields() {
        let key = NodeDedupKey {
            protocol: Protocol::Vmess,
            address: "host=example.com".to_string(),
            port: 443,
            username: Some("user|name".to_string()),
            uuid: None,
            password: Some("pass=word|123".to_string()),
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };

        let output = key.to_string();
        assert!(output.contains("address=16:host=example.com"));
        assert!(output.contains("username=9:user|name"));
        assert!(output.contains("password=13:pass=word|123"));
    }

    #[test]
    fn handles_port_boundary_values() {
        let min_key = NodeDedupKey {
            protocol: Protocol::Trojan,
            address: "example.com".to_string(),
            port: 0,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };
        let max_key = NodeDedupKey {
            port: u16::MAX,
            ..min_key.clone()
        };

        assert!(min_key.to_string().contains("port=1:0"));
        assert!(max_key.to_string().contains("port=5:65535"));
        assert_ne!(min_key.to_string(), max_key.to_string());
    }

    #[test]
    fn covers_all_protocol_variants() {
        let protocols = vec![
            (Protocol::Vless, "vless"),
            (Protocol::Vmess, "vmess"),
            (Protocol::Ss, "ss"),
            (Protocol::Trojan, "trojan"),
            (Protocol::Http, "http"),
            (Protocol::Socks5, "socks5"),
            (Protocol::Hy2, "hy2"),
        ];

        for (protocol, expected_name) in protocols {
            let key = NodeDedupKey {
                protocol: protocol.clone(),
                address: "example.com".to_string(),
                port: 443,
                username: None,
                uuid: None,
                password: None,
                method: None,
                network: "tcp".to_string(),
                tls: None,
                sni: None,
                host: None,
                path: None,
            };

            let output = key.to_string();
            let expected = format!("protocol={}:{}", expected_name.len(), expected_name);
            assert!(
                output.contains(&expected),
                "Protocol {:?} should produce {}",
                protocol,
                expected
            );
        }
    }

    #[test]
    fn produces_identical_keys_for_equivalent_configs() {
        let key1 = NodeDedupKey {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("test-uuid".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("sni.example.com".to_string()),
            host: Some("host.example.com".to_string()),
            path: Some("/path".to_string()),
        };

        let key2 = key1.clone();

        assert_eq!(key1.to_string(), key2.to_string());
    }

    #[test]
    fn distinguishes_different_network_types() {
        let base = NodeDedupKey {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };

        let ws_key = NodeDedupKey {
            network: "ws".to_string(),
            ..base.clone()
        };
        let grpc_key = NodeDedupKey {
            network: "grpc".to_string(),
            ..base.clone()
        };

        assert_ne!(base.to_string(), ws_key.to_string());
        assert_ne!(base.to_string(), grpc_key.to_string());
        assert_ne!(ws_key.to_string(), grpc_key.to_string());
    }

    #[test]
    fn handles_all_optional_fields_present() {
        let key = NodeDedupKey {
            protocol: Protocol::Ss,
            address: "example.com".to_string(),
            port: 8388,
            username: Some("user".to_string()),
            uuid: Some("uuid".to_string()),
            password: Some("pass".to_string()),
            method: Some("aes-256-gcm".to_string()),
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("sni.example.com".to_string()),
            host: Some("host.example.com".to_string()),
            path: Some("/path".to_string()),
        };

        let output = key.to_string();
        assert!(!output.contains("=-"), "No field should be None");
        assert!(output.contains("username=4:user"));
        assert!(output.contains("uuid=4:uuid"));
        assert!(output.contains("password=4:pass"));
        assert!(output.contains("method=11:aes-256-gcm"));
    }

    #[test]
    fn handles_all_optional_fields_absent() {
        let key = NodeDedupKey {
            protocol: Protocol::Http,
            address: "example.com".to_string(),
            port: 80,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
        };

        let output = key.to_string();
        let none_count = output.matches("=-").count();
        assert_eq!(none_count, 8, "Should have 8 None fields");
    }
}
