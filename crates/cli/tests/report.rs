use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn report_selects_registry_and_all_keeps_each_report() {
    let root = std::env::temp_dir().join(format!(
        "ayeneh-report-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let data = root.join("data");
    fs::create_dir_all(&data).unwrap();
    for registry in ["pypi", "npm", "gradle"] {
        fs::write(
            data.join(format!("{registry}.json")),
            r#"{"package":"sample","mirrors":[]}"#,
        )
        .unwrap();
    }

    for (selection, expected) in [
        ("pypi", vec!["pypi"]),
        ("npm", vec!["npm"]),
        ("gradle", vec!["gradle"]),
        ("all", vec!["gradle", "npm", "pypi"]),
        ("", vec!["gradle", "npm", "pypi"]),
    ] {
        let reports = root.join(if selection.is_empty() {
            "default"
        } else {
            selection
        });
        let mut command = Command::new(env!("CARGO_BIN_EXE_ayeneh-cli"));
        command.arg("report");
        if !selection.is_empty() {
            command.arg(selection);
        }
        let output = command
            .env("AYN_DATA_DIR", &data)
            .env("AYN_REPORTS_DIR", &reports)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut registries: Vec<_> = fs::read_dir(reports)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let content = fs::read_to_string(&path).unwrap();
                let name = path.file_name().unwrap().to_string_lossy();
                let registry = name.split('_').next().unwrap().to_string();
                assert!(content.contains(&format!("\"registry\": \"{registry}\"")));
                registry
            })
            .collect();
        registries.sort();
        assert_eq!(registries, expected);
    }
    let invalid = Command::new(env!("CARGO_BIN_EXE_ayeneh-cli"))
        .args(["report", "unknown"])
        .output()
        .unwrap();
    assert!(!invalid.status.success());
    fs::remove_dir_all(root).unwrap();
}
