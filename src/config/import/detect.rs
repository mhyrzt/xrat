use super::ImportMode;

pub fn detect_format(input: &str) -> ImportMode {
    let trimmed = input.trim();

    if trimmed.starts_with('{') {
        if trimmed.contains("\"version\"") || trimmed.contains("\"inbounds\"") {
            return ImportMode::XrayJson;
        }
        return ImportMode::Sip008Json;
    }

    if !trimmed.contains('\n') {
        if is_share_link(trimmed) {
            return ImportMode::SingleLink;
        }
        return ImportMode::Base64Subscription;
    }

    let first_line = trimmed.lines().next().unwrap_or("");
    if is_share_link(first_line) {
        return ImportMode::PlainList;
    }

    ImportMode::Base64Subscription
}

pub fn is_share_link(line: &str) -> bool {
    let line = line.trim();
    line.starts_with("vless://")
        || line.starts_with("vmess://")
        || line.starts_with("ss://")
        || line.starts_with("trojan://")
        || line.starts_with("http://")
        || line.starts_with("https://")
        || line.starts_with("socks5://")
}
