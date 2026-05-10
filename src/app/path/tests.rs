use super::{AppPaths, ensure_config_file, ensure_layout_at, resolve_root_dir_from};

#[test]
fn uses_xrat_path_when_present() {
    let resolved =
        resolve_root_dir_from(Some("/tmp/custom-xrat".into()), Some("/home/tester".into()))
            .expect("path should resolve");

    assert_eq!(resolved, std::path::PathBuf::from("/tmp/custom-xrat"));
}

#[test]
fn falls_back_to_home_config_directory() {
    let resolved =
        resolve_root_dir_from(None, Some("/home/tester".into())).expect("path should resolve");

    assert_eq!(
        resolved,
        std::path::PathBuf::from("/home/tester/.config/xrat")
    );
}

#[test]
fn ensures_layout_and_creates_default_config_file() {
    let root_dir = std::env::temp_dir().join(format!(
        "xrat-app-layout-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ));

    let paths = ensure_layout_at(&root_dir).expect("layout should be created");

    assert_eq!(
        paths,
        AppPaths {
            root_dir: root_dir.clone(),
            database_path: root_dir.join("db.sqlite"),
            config_path: root_dir.join("config.toml"),
        }
    );
    assert!(paths.root_dir.is_dir());
    assert!(paths.config_path.is_file());

    let config = std::fs::read_to_string(&paths.config_path).expect("config should exist");
    assert!(config.contains("XRAT configuration"));

    let _ = std::fs::remove_file(paths.config_path);
    let _ = std::fs::remove_dir(paths.root_dir);
}

#[test]
fn ensures_overridden_config_file_parent_and_file_exist() {
    let root_dir = std::env::temp_dir().join(format!(
        "xrat-config-override-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be valid")
            .as_nanos()
    ));
    let config_path = root_dir.join("nested").join("custom.toml");

    ensure_config_file(&config_path).expect("config file should be created");

    assert!(config_path.is_file());
    let config = std::fs::read_to_string(&config_path).expect("config should exist");
    assert!(config.contains("XRAT configuration"));

    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(root_dir.join("nested"));
    let _ = std::fs::remove_dir(root_dir);
}
