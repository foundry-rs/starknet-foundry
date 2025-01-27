use super::*;
use super::common::runner::{setup_package, test_runner};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_clean_coverage() {
    // Setup: Create a temporary directory and a coverage directory
    let temp = tempdir().unwrap();
    let package_root = temp.path().to_path_buf();
    let coverage_dir = package_root.join("coverage");
    fs::create_dir(&coverage_dir).unwrap();

    // Action: Call the clean_coverage function
    clean_coverage(&vec![package_root.clone()]).unwrap();

    // Assert: Verify the coverage directory is removed
    assert!(!coverage_dir.exists());
}

#[test]
fn test_clean_profile() {
    // Setup: Create a temporary directory and a profile directory
    let temp = tempdir().unwrap();
    let package_root = temp.path().to_path_buf();
    let profile_dir = package_root.join("profile");
    fs::create_dir(&profile_dir).unwrap();

    // Action: Call the clean_profile function
    clean_profile(&vec![package_root.clone()]).unwrap();

    // Assert: Verify the profile directory is removed
    assert!(!profile_dir.exists());
}

#[test]
fn test_clean_cache() {
    // Setup: Create a temporary directory and cache/trace directories
    let temp = tempdir().unwrap();
    let cache_dir = temp.path().join(".snfoundry_cache");
    let trace_dir = temp.path().join(".snfoundry_trace");
    fs::create_dir(&cache_dir).unwrap();
    fs::create_dir(&trace_dir).unwrap();

    // Action: Call the clean_cache function
    clean_cache(&cache_dir, &trace_dir).unwrap();

    // Assert: Verify the cache and trace directories are removed
    assert!(!cache_dir.exists());
    assert!(!trace_dir.exists());
}

#[test]
fn test_clean_all() {
    // Setup: Create a temporary directory and all relevant directories
    let temp = tempdir().unwrap();
    let package_root = temp.path().to_path_buf();
    let coverage_dir = package_root.join("coverage");
    let profile_dir = package_root.join("profile");
    let cache_dir = package_root.join(".snfoundry_cache");
    let trace_dir = package_root.join(".snfoundry_trace");

    fs::create_dir(&coverage_dir).unwrap();
    fs::create_dir(&profile_dir).unwrap();
    fs::create_dir(&cache_dir).unwrap();
    fs::create_dir(&trace_dir).unwrap();

    // Action: Call the clean function with CleanComponent::All
    let args = CleanArgs {
        clean_components: vec![CleanComponent::All],
    };
    clean(args).unwrap();

    // Assert: Verify all directories are removed
    assert!(!coverage_dir.exists());
    assert!(!profile_dir.exists());
    assert!(!cache_dir.exists());
    assert!(!trace_dir.exists());
}

#[test]
fn test_clean_specific_component() {
    // Setup: Create a temporary directory and specific directories
    let temp = tempdir().unwrap();
    let package_root = temp.path().to_path_buf();
    let coverage_dir = package_root.join("coverage");
    let profile_dir = package_root.join("profile");

    fs::create_dir(&coverage_dir).unwrap();
    fs::create_dir(&profile_dir).unwrap();

    // Action: Call the clean function with CleanComponent::Coverage
    let args = CleanArgs {
        clean_components: vec![CleanComponent::Coverage],
    };
    clean(args).unwrap();

    // Assert: Verify only the coverage directory is removed
    assert!(!coverage_dir.exists());
    assert!(profile_dir.exists());
}