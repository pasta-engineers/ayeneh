use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[test]
fn schedule_selects_registries_each_cycle() {
    let root = std::env::temp_dir().join(format!(
        "ayeneh-schedule-test-{}-{}",
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
        command.arg("schedule");
        if !selection.is_empty() {
            command.arg(selection);
        }
        let mut child = command
            .env("AYN_DATA_DIR", &data)
            .env("AYN_REPORTS_DIR", &reports)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if line.unwrap().starts_with("Sleeping for ") {
                    let _ = sender.send(());
                    break;
                }
            }
        });
        let cycle = receiver.recv_timeout(Duration::from_secs(10));
        child.kill().unwrap();
        child.wait().unwrap();
        reader.join().unwrap();
        assert!(
            cycle.is_ok(),
            "scheduler did not finish a cycle for {selection}"
        );
        let mut registries: Vec<_> = fs::read_dir(reports)
            .unwrap()
            .map(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .split('_')
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect();
        registries.sort();
        assert_eq!(registries, expected);
    }
    assert!(!Command::new(env!("CARGO_BIN_EXE_ayeneh-cli"))
        .args(["schedule", "unknown"])
        .output()
        .unwrap()
        .status
        .success());
    fs::remove_dir_all(root).unwrap();
}
