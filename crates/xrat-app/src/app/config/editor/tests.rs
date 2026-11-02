use super::prelude::*;
use super::session::*;
use super::types::*;

fn session(contents: &str) -> (tempfile::TempDir, ConfigEditSession) {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, contents).expect("config should be written");
    let session = ConfigEditSession::open(&path).expect("session should open");
    (root, session)
}

#[test]
fn updates_runtime_binary_path_without_losing_comments() {
    let root = tempfile::tempdir().expect("temp directory should be created");
    let path = root.path().join("config.toml");
    fs::write(&path, "# keep me\n[runtime]\nengine = \"xray\"\n")
        .expect("config should be written");

    update_runtime_binary_path(&path, "xray", Path::new("/tmp/managed/xray"))
        .expect("path should update");

    let contents = fs::read_to_string(path).expect("config should be readable");
    assert!(contents.contains("# keep me"));
    assert!(contents.contains("xray = \"/tmp/managed/xray\""));
}

#[test]
fn exposes_supported_roots_and_effective_defaults() {
    let (_root, session) = session("# keep me\n[database]\nbackend = \"sqlite\"\n");
    let editable_roots = [
        "runtime",
        "subscriptions",
        "routing",
        "dns",
        "testing",
        "server",
        "parser",
        "tui",
    ];

    assert!(session.settings.iter().all(|setting| {
        editable_roots
            .iter()
            .any(|root| setting.path.starts_with(root))
    }));
    assert!(
        session
            .settings
            .iter()
            .any(|setting| setting.path == "tui.theme"
                && setting.value == SettingValue::Text("default".to_string()))
    );
    assert!(
        session
            .settings
            .iter()
            .any(|setting| setting.path == "tui.mouse"
                && setting.value == SettingValue::Bool(true))
    );
    assert!(
        session
            .settings
            .iter()
            .any(|setting| setting.path == "runtime.socks.port"
                && setting.value == SettingValue::Integer(18200))
    );
    assert!(
        !session
            .settings
            .iter()
            .any(|setting| setting.path.starts_with("database."))
    );
    assert!(
        !session
            .settings
            .iter()
            .any(|setting| setting.path.starts_with("dns.hosts."))
    );
}

#[test]
fn help_metadata_covers_every_exposed_setting() {
    let (_root, session) = session("");
    let missing: Vec<&str> = session
        .settings
        .iter()
        .filter(|setting| super::help::for_path(&setting.path).is_none())
        .map(|setting| setting.path.as_str())
        .collect();
    assert!(missing.is_empty(), "missing setting help for {missing:?}");

    for setting in &session.settings {
        assert!(
            !setting.help.description.trim().is_empty(),
            "{}",
            setting.path
        );
        assert!(setting.help.example.contains(" = "), "{}", setting.path);
        assert!(
            !setting.possible_values().trim().is_empty(),
            "{}",
            setting.path
        );
        if let SettingKind::Enum(options) = &setting.kind {
            let possible_values = setting.possible_values();
            assert!(
                options
                    .iter()
                    .all(|option| possible_values.contains(option.as_str())),
                "{}",
                setting.path
            );
        }
    }
}

#[test]
fn settings_use_concise_humanized_labels() {
    let (_root, session) = session("");
    let label = |path: &str| {
        session
            .settings
            .iter()
            .find(|setting| setting.path == path)
            .unwrap_or_else(|| panic!("missing setting {path}"))
            .label
            .as_str()
    };

    assert_eq!(label("parser.parse_mode"), "Mode");
    assert_eq!(label("dns.query_strategy"), "Query strategy");
    assert_eq!(label("runtime.fragment.packets_mode"), "Packet mode");
    assert_eq!(label("runtime.log.dns_log"), "DNS logging");
    assert_eq!(label("runtime.mux.xudp_proxy_udp443"), "UDP 443 handling");
    assert_eq!(
        label("runtime.rotation.health_failure_threshold"),
        "Failure threshold"
    );
    assert_eq!(label("server.pac_allowed_hosts"), "PAC allowed hosts");
    assert_eq!(label("testing.geoip.remote.timeout_ms"), "Timeout");
    assert_eq!(label("testing.geoip.cache.ttl_secs"), "TTL");
}

#[test]
fn settings_expose_origin_and_default_metadata() {
    let (_root, session) = session("[runtime.socks]\nport = 1080\n");
    let port = session
        .settings
        .iter()
        .find(|setting| setting.path == "runtime.socks.port")
        .expect("port setting");
    let dns = session
        .settings
        .iter()
        .find(|setting| setting.path == "dns.servers")
        .expect("DNS setting");

    assert!(port.is_explicit());
    assert_eq!(port.default_value(), &SettingValue::Integer(18200));
    assert!(!dns.is_explicit());
    assert_eq!(dns.default_value(), &SettingValue::List(Vec::new()));
}

#[test]
fn saves_routing_and_dns_without_touching_dynamic_hosts() {
    let contents = "[dns]\nquery_strategy = \"UseSystem\"\n\
                    [dns.hosts]\n\"domain:example.test\" = \"127.0.0.1\"\n\
                    [routing]\ndomain_strategy = \"IPIfNonMatch\"\n";
    let (root, mut session) = session(contents);
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "dns.query_strategy")
        .expect("DNS strategy setting")
        .cycle_enum(1);
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "routing.direct.domain")
        .expect("direct-domain setting")
        .set_from_input("example.com, domain:internal")
        .expect("domain list should parse");

    let outcome = session.save().expect("save should succeed");
    let saved = fs::read_to_string(root.path().join("config.toml")).expect("config should read");
    assert!(saved.contains("\"domain:example.test\" = \"127.0.0.1\""));
    assert!(saved.contains("query_strategy = \"UseIP\""));
    assert!(saved.contains("domain = [\"example.com\", \"domain:internal\"]"));
    assert_eq!(outcome.config.dns.query_strategy, "UseIP");
    assert_eq!(
        outcome.config.routing.direct.domain,
        ["example.com", "domain:internal"]
    );
    assert_eq!(
        outcome.effects,
        [SettingEffect::RuntimeRestart].into_iter().collect()
    );
}

#[test]
fn saves_only_changed_key_and_preserves_comments() {
    let (root, mut session) = session("# keep me\n[runtime.socks]\nport = 18200\n");
    let setting = session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.socks.port")
        .expect("port setting");
    setting.set_from_input("1080").expect("input should parse");

    let outcome = session.save().expect("save should succeed");
    let saved = fs::read_to_string(root.path().join("config.toml")).expect("config should read");
    assert!(saved.contains("# keep me"));
    assert!(saved.contains("port = 1080"));
    assert_eq!(outcome.config.runtime.socks.port, 1080);
    assert_eq!(outcome.changed_paths, ["runtime.socks.port"]);
}

#[cfg(unix)]
#[test]
fn save_without_changes_does_not_replace_config_file() {
    use std::os::unix::fs::MetadataExt;

    let (root, mut session) = session("[runtime.socks]\nport = 1080\n");
    let path = root.path().join("config.toml");
    let inode = fs::metadata(&path).expect("metadata").ino();

    let outcome = session.save().expect("unchanged save should succeed");

    assert!(outcome.changed_paths.is_empty());
    assert_eq!(fs::metadata(path).expect("metadata").ino(), inode);
}

#[test]
fn reset_removes_explicit_override() {
    let (root, mut session) = session("[runtime.socks]\nport = 1080\n");
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.socks.port")
        .expect("port setting")
        .reset_to_default();

    let outcome = session.save().expect("save should succeed");
    let saved = fs::read_to_string(root.path().join("config.toml")).expect("config should read");
    assert!(!saved.contains("port ="));
    assert_eq!(outcome.config.runtime.socks.port, 18200);
}

#[test]
fn rejects_external_changes_without_overwriting_them() {
    let (root, mut session) = session("[runtime]\nengine = \"xray\"\n");
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.engine")
        .expect("engine setting")
        .cycle_enum(1);
    let path = root.path().join("config.toml");
    fs::write(&path, "# external\n[runtime]\nengine = \"xray\"\n")
        .expect("external edit should write");

    let error = session.save().expect_err("save should reject conflict");
    assert!(error.contains("changed on disk"));
    assert!(
        fs::read_to_string(path)
            .expect("config should read")
            .contains("# external")
    );
}

#[test]
fn semantic_validation_prevents_invalid_cross_field_save() {
    let (root, mut session) = session("# original\n");
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.http.enabled")
        .expect("http enabled setting")
        .toggle();
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.http.port")
        .expect("http port setting")
        .set_from_input("18200")
        .expect("port should parse");

    let error = session.save().expect_err("duplicate ports should fail");
    assert!(error.contains("[runtime.http].port"));
    assert_eq!(
        fs::read_to_string(root.path().join("config.toml")).expect("config should read"),
        "# original\n"
    );
}

#[test]
fn saves_environment_backed_secret_without_exposing_a_literal() {
    let (root, mut session) = session("[server]\nenabled = false\n");
    session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "server.key")
        .expect("server key setting")
        .set_from_input("env:XRAT_SERVER_KEY")
        .expect("secret should parse");

    let outcome = session.save().expect("save should succeed");
    let saved = fs::read_to_string(root.path().join("config.toml")).expect("config should read");
    assert!(saved.contains("key = { env = \"XRAT_SERVER_KEY\" }"));
    assert!(outcome.effects.contains(&SettingEffect::DaemonRestart));
}
