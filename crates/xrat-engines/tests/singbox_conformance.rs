#[path = "singbox_conformance/fixtures.rs"]
mod fixtures;

use std::{collections::BTreeSet, path::PathBuf, sync::Arc};
use xrat_engines::singbox::{config_check_command, ensure_supported_binary};
use xrat_support::process::SystemProcessSpawner;

#[test]
fn fixture_matrix_is_deterministic() {
    let first = fixtures::matrix();
    let second = fixtures::matrix();
    let mut names = BTreeSet::new();
    assert_eq!(first.len(), 211);
    assert_eq!(
        first
            .iter()
            .filter(|(name, _)| name.starts_with("inbounds-"))
            .count(),
        16
    );
    for ((name, config), (other_name, other_config)) in first.iter().zip(&second) {
        assert!(names.insert(name));
        assert_eq!(name, other_name);
        let json = serde_json::to_string_pretty(config).unwrap();
        assert_eq!(json, serde_json::to_string_pretty(other_config).unwrap());
    }
}

#[test]
#[ignore = "native conformance skipped by default; requires XRAT_CONFORMANCE_SINGBOX"]
fn native_singbox_conformance() {
    let binary = PathBuf::from(std::env::var_os("XRAT_CONFORMANCE_SINGBOX").expect(
        "set XRAT_CONFORMANCE_SINGBOX to a supported validator; missing binary is not a pass",
    ));
    ensure_supported_binary(&binary).expect("supported sing-box validator must be available");
    let temporary = tempfile::tempdir().unwrap();
    let directory = std::env::var_os("XRAT_CONFORMANCE_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| temporary.path().to_owned());
    std::fs::create_dir_all(&directory).unwrap();
    let version = xrat_support::process::Command::new(&binary)
        .arg("version")
        .output()
        .unwrap();
    std::fs::write(directory.join("validator-version.txt"), &version.stdout).unwrap();
    let mut results = Vec::new();
    let mut failures = Vec::new();
    for (name, config) in fixtures::matrix() {
        let json = serde_json::to_string_pretty(&config).unwrap();
        let file = directory.join(format!("{name}.json"));
        std::fs::write(&file, &json).unwrap();
        let output = config_check_command(&binary, &file, Arc::new(SystemProcessSpawner))
            .output()
            .unwrap_or_else(|error| panic!("{name}: {error}\n{json}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        results.push(
            serde_json::json!({"fixture":name,"passed":output.status.success(),
            "stdout":stdout,"stderr":stderr}),
        );
        if !output.status.success() {
            failures.push(format!("{name}: {stdout}\n{stderr}\n{json}"));
        }
    }
    std::fs::write(
        directory.join("results.json"),
        serde_json::to_vec_pretty(&results).unwrap(),
    )
    .unwrap();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    println!("native conformance: {} fixtures passed", results.len());
}
