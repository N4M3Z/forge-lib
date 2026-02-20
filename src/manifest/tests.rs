use super::*;
use tempfile::TempDir;

#[test]
fn roundtrip() {
    let dir = TempDir::new().unwrap();
    let entries = vec!["Alpha".to_string(), "Beta".to_string()];
    update(dir.path(), "forge-council", &entries).unwrap();
    let loaded = read(dir.path(), "forge-council");
    assert_eq!(loaded, entries);
}

#[test]
fn read_missing_returns_empty() {
    let dir = TempDir::new().unwrap();
    let loaded = read(dir.path(), "forge-council");
    assert!(loaded.is_empty());
}

#[test]
fn read_wrong_module_returns_empty() {
    let dir = TempDir::new().unwrap();
    let entries = vec!["Alpha".to_string()];
    update(dir.path(), "forge-council", &entries).unwrap();
    let loaded = read(dir.path(), "other-module");
    assert!(loaded.is_empty());
}

#[test]
fn multi_module() {
    let dir = TempDir::new().unwrap();
    let council = vec!["Council".to_string()];
    let other = vec!["Helper".to_string(), "Util".to_string()];
    update(dir.path(), "forge-council", &council).unwrap();
    update(dir.path(), "forge-other", &other).unwrap();
    assert_eq!(read(dir.path(), "forge-council"), council);
    assert_eq!(read(dir.path(), "forge-other"), other);
}

#[test]
fn empty_entries_removes_module() {
    let dir = TempDir::new().unwrap();
    let entries = vec!["Alpha".to_string()];
    update(dir.path(), "forge-council", &entries).unwrap();
    update(dir.path(), "forge-council", &[]).unwrap();
    assert!(read(dir.path(), "forge-council").is_empty());
}

#[test]
fn empty_map_removes_file() {
    let dir = TempDir::new().unwrap();
    let entries = vec!["Alpha".to_string()];
    update(dir.path(), "forge-council", &entries).unwrap();
    assert!(dir.path().join(".manifest").exists());
    update(dir.path(), "forge-council", &[]).unwrap();
    assert!(!dir.path().join(".manifest").exists());
}
