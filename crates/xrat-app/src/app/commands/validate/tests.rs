use super::entry::collect_errors;
use super::prelude::*;

fn errors_for(config: &AppConfig) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    validate_config(config, &mut errors);
    errors
}

fn find<'a>(diagnostics: &'a [Diagnostic], field: &str) -> &'a Diagnostic {
    diagnostics
        .iter()
        .find(|diagnostic| diagnostic.field == field)
        .unwrap_or_else(|| panic!("expected a diagnostic for {field}: {diagnostics:?}"))
}

fn errors_for_toml(contents: &str) -> Vec<Diagnostic> {
    let root_dir = std::env::temp_dir().join(format!(
        "xrat-validate-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    let config_path = root_dir.join("config.toml");
    std::fs::write(&config_path, contents).expect("config should be written");

    let errors = collect_errors(&ValidateArgs {
        path: config_path.clone(),
        format: ValidateFormat::Human,
    });

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
    errors
}

#[test]
fn accepts_default_config() {
    let errors = errors_for(&AppConfig::default());

    assert!(
        errors.is_empty(),
        "default config should validate: {errors:?}"
    );
}

#[test]
fn rejects_invalid_runtime_engine() {
    let mut config = AppConfig::default();
    config.runtime.engine = "bad".to_string();

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime].engine");

    assert!(diagnostic.problem.contains("bad"));
    assert!(diagnostic.fix.contains("xray"));
    assert!(!diagnostic.reason.is_empty());
}

#[test]
fn rejects_tun_with_v2ray_engine() {
    let mut config = AppConfig::default();
    config.runtime.engine = "v2ray".to_string();
    config.runtime.tun.enabled = true;

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime.tun].enabled");

    assert!(diagnostic.problem.contains("v2ray"));
}

#[test]
fn rejects_xray_tun_route_exclusions() {
    let mut config = AppConfig::default();
    config.runtime.tun.enabled = true;
    config.runtime.tun.route_exclude_address = vec!["192.168.0.0/16".to_string()];

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime.tun].route_exclude_address");

    assert!(diagnostic.problem.contains("xray"));
}

#[test]
fn rejects_invalid_tun_settings() {
    let mut config = AppConfig::default();
    config.runtime.engine = "sing-box".to_string();
    config.runtime.tun.enabled = true;
    config.runtime.tun.interface_name = String::new();
    config.runtime.tun.mtu = 100;
    config.runtime.tun.stack = "wireguard".to_string();
    config.runtime.tun.address = vec!["not-a-cidr".to_string()];
    config.runtime.tun.route_exclude_address = vec!["10.0.0.0/33".to_string()];

    let errors = errors_for(&config);

    assert!(
        find(&errors, "[runtime.tun].interface_name")
            .problem
            .contains("empty")
    );
    assert!(find(&errors, "[runtime.tun].mtu").problem.contains("100"));
    assert!(
        find(&errors, "[runtime.tun].stack")
            .problem
            .contains("wireguard")
    );
    assert!(
        find(&errors, "[runtime.tun].address")
            .problem
            .contains("not-a-cidr")
    );
    assert!(
        find(&errors, "[runtime.tun].route_exclude_address")
            .problem
            .contains("10.0.0.0/33")
    );
}

#[test]
fn rejects_invalid_routing_domain_strategy() {
    let mut config = AppConfig::default();
    config.routing.domain_strategy = "AlwaysIP".to_string();

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[routing].domain_strategy");

    assert!(diagnostic.problem.contains("AlwaysIP"));
    assert!(diagnostic.fix.contains("IPIfNonMatch"));
}

#[test]
fn rejects_blank_routing_rules() {
    let mut config = AppConfig::default();
    config.routing.direct.domain = vec!["   ".to_string()];
    config.routing.block.geoip = vec![String::new()];

    let errors = errors_for(&config);

    assert_eq!(
        find(&errors, "[routing.direct].domain").problem,
        "contains an empty routing rule"
    );
    assert_eq!(
        find(&errors, "[routing.block].geoip").problem,
        "contains an empty routing rule"
    );
}

#[test]
fn rejects_invalid_mux_values() {
    let mut config = AppConfig::default();
    config.runtime.mux.concurrency = 200;
    config.runtime.mux.xudp_concurrency = 5000;
    config.runtime.mux.xudp_proxy_udp443 = "nope".to_string();

    let errors = errors_for(&config);
    assert!(
        find(&errors, "[runtime.mux].concurrency")
            .fix
            .contains("128")
    );
    assert!(
        find(&errors, "[runtime.mux].xudp_concurrency")
            .fix
            .contains("1024")
    );
    assert!(
        find(&errors, "[runtime.mux].xudp_proxy_udp443")
            .fix
            .contains("reject")
    );
}

#[test]
fn rejects_invalid_fragment_ranges() {
    let mut config = AppConfig::default();
    config.runtime.fragment.packets_mode = "range".to_string();
    config.runtime.fragment.packets = [0, 3];
    config.runtime.fragment.length = [200, 100];
    config.runtime.fragment.interval = [30, 10];

    let errors = errors_for(&config);
    assert!(
        !find(&errors, "[runtime.fragment].packets")
            .reason
            .is_empty()
    );
    assert!(!find(&errors, "[runtime.fragment].length").reason.is_empty());
    assert!(
        !find(&errors, "[runtime.fragment].interval")
            .reason
            .is_empty()
    );
}

#[test]
fn rejects_invalid_network_bind_address() {
    let mut config = AppConfig::default();
    config.runtime.network.bind_address = "not-an-ip".to_string();
    config.runtime.network.mark = -1;

    let errors = errors_for(&config);
    assert!(
        find(&errors, "[runtime.network].bind_address")
            .problem
            .contains("not-an-ip")
    );
    assert!(!find(&errors, "[runtime.network].mark").reason.is_empty());
}

#[test]
fn rejects_unknown_fragment_packets_mode() {
    let mut config = AppConfig::default();
    config.runtime.fragment.packets_mode = "bogus".to_string();

    let errors = errors_for(&config);
    assert!(
        find(&errors, "[runtime.fragment].packets_mode")
            .fix
            .contains("range")
    );
}

#[test]
fn accepts_valid_fragment_packet_range() {
    let mut config = AppConfig::default();
    config.runtime.fragment.packets_mode = "range".to_string();
    config.runtime.fragment.packets = [1, 3];

    let errors = errors_for(&config);
    assert!(
        !errors
            .iter()
            .any(|diagnostic| diagnostic.field == "[runtime.fragment].packets"),
        "valid packets range should not error: {errors:?}"
    );
}

#[test]
fn rejects_duplicate_enabled_inbound_ports() {
    let mut config = AppConfig::default();
    config.runtime.http.enabled = true;
    config.runtime.http.port = config.runtime.socks.port;

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime.http].port");

    assert!(diagnostic.problem.contains("already used"));
    assert!(diagnostic.fix.contains("1-65535"));
}

#[test]
fn port_zero_reports_range_in_fix() {
    let mut config = AppConfig::default();
    config.runtime.socks.port = 0;

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime.socks].port");

    assert!(diagnostic.fix.contains("1-65535"));
    assert!(!diagnostic.reason.is_empty());
}

#[test]
fn timeout_zero_reports_reason_and_fix() {
    let mut config = AppConfig::default();
    config.testing.tcp.enabled = true;
    config.testing.tcp.timeout = 0;

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[testing.tcp].timeout");

    assert!(!diagnostic.reason.is_empty());
    assert!(diagnostic.fix.contains("seconds"));
}

#[test]
fn invalid_test_url_reports_scheme_guidance() {
    let mut config = AppConfig::default();
    config.testing.download.enabled = true;
    config.testing.download.url = "ftp://example.com".to_string();

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[testing.download].url");

    assert!(diagnostic.fix.contains("http"));
}

#[test]
fn rejects_out_of_range_real_delay_status_code() {
    let mut config = AppConfig::default();
    config.testing.real_delay.accepted_status_codes = Some(vec![99, 600]);

    let errors = errors_for(&config);
    let diagnostics = errors
        .iter()
        .filter(|diagnostic| diagnostic.field == "[testing.real_delay].accepted_status_codes")
        .collect::<Vec<_>>();

    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.fix.contains("599"))
    );
}

#[test]
fn rejects_empty_real_delay_status_acceptance() {
    let errors = errors_for_toml(
        "[testing.real_delay]\naccepted_status_codes = []\naccepted_status_ranges = []\n",
    );
    let diagnostic = find(&errors, "[testing.real_delay]");

    assert!(diagnostic.problem.contains("empty"));
    assert!(diagnostic.fix.contains("omit both"));
}

#[test]
fn rejects_malformed_real_delay_status_range_with_guidance() {
    let errors = errors_for_toml("[testing.real_delay]\naccepted_status_ranges = [\"399-300\"]\n");
    let diagnostic = find(&errors, "[testing.real_delay].accepted_status_ranges");

    assert!(diagnostic.problem.contains("starts after it ends"));
    assert!(diagnostic.fix.contains("300-399"));
}

#[test]
fn accepts_rotation_stage_aliases() {
    let mut config = AppConfig::default();
    config.runtime.rotation.test_stages = vec![
        "ping".to_string(),
        "real-delay".to_string(),
        "download".to_string(),
    ];

    let errors = errors_for(&config);

    assert!(
        !errors
            .iter()
            .any(|diagnostic| diagnostic.field == "[runtime.rotation].test_stages"),
        "stage aliases should be accepted: {errors:?}"
    );
}

#[test]
fn rejects_unknown_rotation_stage() {
    let mut config = AppConfig::default();
    config.runtime.rotation.test_stages = vec!["bogus".to_string()];

    let errors = errors_for(&config);
    let diagnostic = find(&errors, "[runtime.rotation].test_stages");

    assert!(diagnostic.problem.contains("bogus"));
    assert!(diagnostic.fix.contains("icmp"));
}

#[test]
fn env_secret_does_not_require_resolution() {
    let mut config = AppConfig::default();
    config.server.enabled = true;
    config.server.key = Some(SecretString::Env {
        env: "XRAT_UNSET_VALIDATE_KEY".to_string(),
    });

    let errors = errors_for(&config);

    assert!(
        !errors
            .iter()
            .any(|diagnostic| diagnostic.field == "[server].key"),
        "env-referenced secret should pass structural validation: {errors:?}"
    );
}

#[test]
fn rejects_missing_config_file() {
    let args = ValidateArgs {
        path: "/tmp/xrat-missing-config.toml".into(),
        format: ValidateFormat::Human,
    };

    let errors = collect_errors(&args);

    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.problem.contains("config file does not exist"))
    );
}

#[test]
fn rejects_unknown_test_stage_before_deserializing() {
    let errors = errors_for_toml(
        r#"
        [testing]
        order = ["icmp", "bogus"]
        "#,
    );

    let diagnostic = find(&errors, "[testing].order");
    assert!(diagnostic.problem.contains("bogus"));
    assert!(diagnostic.fix.contains("icmp"));
    assert!(diagnostic.fix.contains("tcp"));
}

#[test]
fn rejects_unknown_failure_policy_before_deserializing() {
    let errors = errors_for_toml(
        r#"
        [testing]
        failure_policy = "skip"
        "#,
    );

    let diagnostic = find(&errors, "[testing].failure_policy");
    assert!(diagnostic.problem.contains("skip"));
    assert!(diagnostic.fix.contains("skip_remaining"));
    assert!(diagnostic.fix.contains("alias"));
}

#[test]
fn rejects_string_typed_duration_before_deserializing() {
    let errors = errors_for_toml(
        r#"
        [testing.tcp]
        timeout = "2000"
        "#,
    );

    let diagnostic = find(&errors, "[testing.tcp].timeout");
    assert!(diagnostic.problem.contains("expected integer milliseconds"));
    assert!(diagnostic.problem.contains("a string"));
}

#[test]
fn rejects_unknown_database_backend_before_deserializing() {
    let errors = errors_for_toml(
        r#"
        [database]
        backend = "mysql"
        "#,
    );

    let diagnostic = find(&errors, "[database].backend");
    assert!(diagnostic.problem.contains("mysql"));
    assert!(diagnostic.fix.contains("postgres"));
}

#[test]
fn rejects_unknown_geoip_backend_before_deserializing() {
    let errors = errors_for_toml(
        r#"
        [testing.geoip]
        backend = "bogus"
        "#,
    );

    let diagnostic = find(&errors, "[testing.geoip].backend");
    assert!(diagnostic.problem.contains("bogus"));
    assert!(diagnostic.fix.contains("mmdb"));
}

#[test]
fn reports_invalid_toml_syntax_as_structural_error() {
    let errors = errors_for_toml("[testing\norder = [\"icmp\"]\n");

    assert!(
        errors
            .iter()
            .any(|diagnostic| diagnostic.problem.contains("not valid TOML")),
        "expected a syntax diagnostic: {errors:?}"
    );
}

#[test]
fn accepts_valid_config_written_to_disk() {
    let errors = errors_for_toml(
        r#"
        [testing]
        order = ["icmp", "tcp", "real_delay"]
        failure_policy = "skip_remaining"

        [testing.tcp]
        enabled = true
        timeout = 2000
        "#,
    );

    assert!(
        errors.is_empty(),
        "valid config should have no errors: {errors:?}"
    );
}

#[test]
fn accepts_valid_tui_theme() {
    let errors = errors_for_toml(
        r#"
        [tui]
        theme = "catppuccin"
        border_style = "rounded"
        transparent = true
        "#,
    );
    assert!(
        errors.is_empty(),
        "valid theme should have no errors: {errors:?}"
    );
}

#[test]
fn rejects_invalid_tui_transparent() {
    let errors = errors_for_toml(
        r#"
        [tui]
        transparent = "yes"
        "#,
    );
    let diagnostic = find(&errors, "[tui].transparent");
    assert!(diagnostic.problem.contains("boolean"));
}

#[test]
fn rejects_invalid_tui_theme() {
    let errors = errors_for_toml(
        r#"
        [tui]
        theme = "neon-glow"
        "#,
    );
    let diagnostic = find(&errors, "[tui].theme");
    assert!(diagnostic.problem.contains("neon-glow"));
    assert!(diagnostic.fix.contains("catppuccin"));
}

#[test]
fn accepts_named_custom_theme_and_legacy_ui_alias() {
    for section in ["tui", "ui"] {
        let input = format!(
            "[{section}]\ntheme = 'custom'\n[{section}.themes.custom]\nextends = 'catppuccin'\naccent = '#123456'"
        );
        assert!(errors_for_toml(&input).is_empty(), "{input}");
    }
}

#[test]
fn rejects_invalid_custom_theme_colors_names_and_bases() {
    for (input, problem) in [
        ("[tui.themes.custom]\naccent = '#12345g'", "accent"),
        ("[tui.themes.nord]", "reserved"),
        ("[tui.themes.custom]\nextends = 'custom'", "extends"),
        ("[tui.themes.custom]\nacccent = 'red'", "acccent"),
    ] {
        let errors = errors_for_toml(input);
        assert!(!errors.is_empty(), "{input}");
        assert!(
            errors[0].problem.contains(problem) || errors[0].field.contains(problem),
            "{errors:?}"
        );
    }
}

#[test]
fn theme_validation_normalizes_builtin_aliases_like_configuration_loading() {
    assert!(errors_for_toml("[tui]\ntheme = ' MoCha '").is_empty());
}

#[test]
fn custom_color_diagnostic_identifies_exact_field() {
    let errors = errors_for_toml("[tui.themes.custom]\naccent = '#12345g'");
    let diagnostic = find(&errors, "[tui.themes.custom].accent");
    assert!(diagnostic.problem.contains("#12345g"));
}
