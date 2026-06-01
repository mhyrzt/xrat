use crate::app::config::{load, resolve_config_path};

#[test]
fn resolves_relative_paths_from_config_directory() {
    let resolved = resolve_config_path("/tmp/xrat/config.toml".as_ref(), "db.sqlite".as_ref());

    assert_eq!(resolved, std::path::PathBuf::from("/tmp/xrat/db.sqlite"));
}

#[test]
fn loads_config_file() {
    let root_dir = std::env::temp_dir().join(format!(
        "xrat-app-config-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ));
    let config_path = root_dir.join("config.toml");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    std::fs::write(&config_path, "[runtime.socks]\nport = 1090\n")
        .expect("config should be written");

    let config = load(&config_path).expect("config should load");

    assert_eq!(config.runtime.socks.port, 1090);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}
