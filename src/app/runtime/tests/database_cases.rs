use super::*;

#[test]
fn resolves_database_from_config_file() {
    let root_dir = temp_root("xrat-runtime-config-db");
    let config_path = root_dir.join("config.toml");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    std::fs::write(&config_path, "[paths]\ndatabase = \"state/db.sqlite\"\n")
        .expect("config should be written");

    let cli = cli_for_config(&config_path);
    let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

    assert_eq!(
        runtime_paths.database_path,
        root_dir.join("state/db.sqlite")
    );
    assert_eq!(runtime_paths.xray_path, PathBuf::from("xray"));
    assert_eq!(runtime_paths.v2ray_path, PathBuf::from("v2ray"));
    assert_eq!(runtime_paths.sing_box_path, PathBuf::from("sing-box"));

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}

#[test]
fn resolves_postgres_database_from_config_file() {
    let root_dir = temp_root("xrat-runtime-config-postgres");
    let config_path = root_dir.join("config.toml");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    std::fs::write(
            &config_path,
            "[database]\nbackend = \"postgres\"\n\n[database.postgres]\nuser = \"xrat user\"\npassword = \"secret/pass\"\nhost = \"db.local\"\nport = 5544\ndb_name = \"xrat db\"\n",
        )
        .expect("config should be written");

    let cli = cli_for_config(&config_path);
    let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

    match runtime_paths.database_config {
        crate::db::DatabaseConnectionConfig::Postgres { url, .. } => {
            assert_eq!(
                url,
                "postgres://xrat%20user:secret%2Fpass@db.local:5544/xrat%20db"
            );
        }
        crate::db::DatabaseConnectionConfig::Sqlite { .. } => {
            panic!("expected postgres config")
        }
    }
    assert_eq!(
        runtime_paths.database_label,
        "postgres://xrat%20user:<redacted>@db.local:5544/xrat%20db"
    );

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}

#[test]
fn cli_database_overrides_config_database() {
    let root_dir = temp_root("xrat-runtime-cli-db");
    let config_path = root_dir.join("config.toml");
    let cli_database = root_dir.join("override.sqlite");
    std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
    std::fs::write(&config_path, "[paths]\ndatabase = \"state/db.sqlite\"\n")
        .expect("config should be written");

    let cli = Cli::parse_from([
        "xrat",
        "--config",
        config_path.to_str().unwrap(),
        "--database",
        cli_database.to_str().unwrap(),
        "list",
        "configs",
    ]);
    let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

    assert_eq!(runtime_paths.database_path, cli_database);

    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_dir(root_dir);
}
