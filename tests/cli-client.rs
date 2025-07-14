use assert_cmd::prelude::*;
use kvs::{KvStore, Result, StoreTrait};
use predicates::ord::eq;
use predicates::str::{PredicateStrExt, contains, is_empty};
use std::fs::{self, File};
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

// `kvs` with no args should exit with a non-zero code.
#[test]
fn client_cli_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.current_dir(&temp_dir).assert().failure();
}

// `kvs -V` should print the version
#[test]
fn client_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-client").unwrap();
    cmd.args(["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

// `kvs get <KEY>` should print "Key not found" for a non-existent key and exit with zero.
#[test]
fn cli_get_non_existent_key() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("Key not found").trim());
}

// `kvs rm <KEY>` should print "Key not found" for an empty database and exit with non-zero code.
#[test]
fn cli_rm_non_existent_key() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(eq("Key not found").trim());
}

// `kvs set <KEY> <VALUE>` should print nothing and exit with zero.
#[test]
fn cli_set() {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["set", "key1", "value1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());
}

#[test]
fn cli_get_stored() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    let mut store = KvStore::open(temp_dir.path())?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;
    drop(store);

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value1").trim());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key2"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("value2").trim());

    Ok(())
}

// `kvs rm <KEY>` should print nothing and exit with zero.
#[test]
fn cli_rm_stored() -> Result<()> {
    let temp_dir = TempDir::new().expect("unable to create temporary working directory");

    let mut store = KvStore::open(temp_dir.path())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    drop(store);

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(is_empty());

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key1"])
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(eq("Key not found").trim());

    Ok(())
}

#[test]
fn client_cli_invalid_get() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "extra", "field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_set() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["set"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["set", "missing_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["set", "key", "value", "extra_field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["set", "key", "value", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["get", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_rm() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm", "extra", "field"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm", "key", "--addr", "invalid-addr"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["rm", "key", "--unknown-flag"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn client_cli_invalid_subcommand() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("kvs-client")
        .unwrap()
        .args(["unknown"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

// `kvs-server -V` should print the version
#[test]
fn server_cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    cmd.args(["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_log_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let stderr_path = temp_dir.path().join("stderr");
    let mut cmd = Command::cargo_bin("kvs-server").unwrap();
    let mut child = cmd
        .args(["--engine", "kvs", "--addr", "127.0.0.1:4001"])
        .current_dir(&temp_dir)
        .stderr(File::create(&stderr_path).unwrap())
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_secs(1));
    child.kill().expect("server exited before killed");

    let content = fs::read_to_string(&stderr_path).expect("unable to read from stderr file");
    assert!(content.contains(env!("CARGO_PKG_VERSION")));
    assert!(content.contains("kvs"));
    assert!(content.contains("127.0.0.1:4001"));
}

#[test]
fn cli_wrong_engine() {
    // sled first, kvs second
    {
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        let mut child = cmd
            .args(["--engine", "sled", "--addr", "127.0.0.1:4002"])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        child.kill().expect("server exited before killed");

        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        cmd.args(["--engine", "kvs", "--addr", "127.0.0.1:4003"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }

    // kvs first, sled second
    {
        let temp_dir = TempDir::new().unwrap();
        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        let mut child = cmd
            .args(["--engine", "kvs", "--addr", "127.0.0.1:4002"])
            .current_dir(&temp_dir)
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        child.kill().expect("server exited before killed");

        let mut cmd = Command::cargo_bin("kvs-server").unwrap();
        cmd.args(["--engine", "sled", "--addr", "127.0.0.1:4003"])
            .current_dir(&temp_dir)
            .assert()
            .failure();
    }
}
