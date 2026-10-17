use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU8, AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::round_seed::{RoundSeedError, RoundSeedSource};

struct TestFile {
    path: PathBuf,
}

impl TestFile {
    fn new(contents: &[u8]) -> Self {
        let path = unique_temp_path();
        fs::write(&path, contents).expect("write seed fixture");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new() -> Self {
        let path = unique_temp_path();
        fs::create_dir(&path).expect("create directory fixture");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn unique_temp_path() -> PathBuf {
    static NEXT_PATH: AtomicUsize = AtomicUsize::new(0);

    let unique = NEXT_PATH.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "grenadiers-round-seed-{}-{nanos}-{unique}",
        std::process::id()
    ))
}

impl Drop for TestFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[test]
fn random_mode_requests_fresh_entropy_only_when_a_seed_is_requested() {
    let calls = Arc::new(AtomicU8::new(0));
    let observed_calls = Arc::clone(&calls);
    let source = RoundSeedSource::with_entropy(move |bytes| {
        let value = observed_calls.fetch_add(1, Ordering::SeqCst) + 1;
        bytes.fill(value);
        Ok(())
    });

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(source.next_seed().expect("first seed"), [1; 32]);
    assert_eq!(source.next_seed().expect("second seed"), [2; 32]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn random_mode_returns_a_redacted_typed_entropy_error() {
    let source = RoundSeedSource::with_entropy(|_| Err(()));

    let error = source.next_seed().expect_err("entropy should fail");

    assert_eq!(error, RoundSeedError::EntropyUnavailable);
    assert_eq!(error.to_string(), "round seed entropy is unavailable");
}

#[test]
fn absent_file_path_uses_operating_system_entropy() {
    let source = RoundSeedSource::from_optional_path(None).expect("construct random source");

    source.next_seed().expect("operating system seed");
}

#[test]
fn fixed_file_accepts_lowercase_hex_with_optional_single_line_feed() {
    let expected = std::array::from_fn(|index| index as u8);
    for contents in [
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\n".as_slice(),
    ] {
        let file = TestFile::new(contents);
        let source = RoundSeedSource::from_optional_path(Some(file.path())).expect("valid seed");

        assert_eq!(source.next_seed().expect("fixed seed"), expected);
        assert_eq!(source.next_seed().expect("repeated fixed seed"), expected);
    }
}

#[test]
fn fixed_file_is_loaded_once() {
    let first = b"1111111111111111111111111111111111111111111111111111111111111111";
    let second = b"2222222222222222222222222222222222222222222222222222222222222222";
    let file = TestFile::new(first);
    let source = RoundSeedSource::from_optional_path(Some(file.path())).expect("valid seed");

    fs::write(file.path(), second).expect("replace seed fixture");

    assert_eq!(source.next_seed().expect("loaded seed"), [0x11; 32]);
}

#[test]
fn fixed_file_rejects_noncanonical_or_oversized_input() {
    for contents in [
        b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1g".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f ".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\r\n".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\n\n".as_slice(),
        b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e".as_slice(),
    ] {
        let file = TestFile::new(contents);

        assert_eq!(
            RoundSeedSource::from_optional_path(Some(file.path()))
                .expect_err("invalid seed should fail"),
            RoundSeedError::InvalidFile
        );
    }
}

#[test]
fn missing_file_returns_redacted_read_error() {
    let secret_path = std::env::temp_dir().join(format!(
        "grenadiers-secret-seed-that-does-not-exist-{}",
        std::process::id()
    ));

    let error = RoundSeedSource::from_optional_path(Some(&secret_path))
        .expect_err("missing file should fail");
    let diagnostics = format!("{error:?} {error}");

    assert_eq!(error, RoundSeedError::FileRead);
    assert!(!diagnostics.contains(secret_path.to_string_lossy().as_ref()));
}

#[test]
fn unreadable_file_returns_typed_read_error() {
    let directory = TestDirectory::new();

    let error = RoundSeedSource::from_optional_path(Some(&directory.path))
        .expect_err("a directory cannot be read as a seed file");

    assert_eq!(error, RoundSeedError::FileRead);
}

#[test]
fn source_debug_output_redacts_fixed_seed_and_file_path() {
    let secret = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let file = TestFile::new(secret.as_bytes());
    let source = RoundSeedSource::from_optional_path(Some(file.path())).expect("valid seed");

    let diagnostics = format!("{source:?}");

    assert!(diagnostics.contains("Fixed(<redacted>)"));
    assert!(!diagnostics.contains(secret));
    assert!(!diagnostics.contains(file.path().to_string_lossy().as_ref()));
}
