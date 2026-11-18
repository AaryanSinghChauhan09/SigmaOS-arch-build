// Integration test for SigmaOS-arch-build
// Verifies the crate builds and links as a dependency.

use sigmaos_arch_build::{BuildConfig, BuildSystem, BuildType};

#[test]
fn crate_links_and_runs() {
    // Arrange
    let mut sys = BuildSystem::new();
    let config = BuildConfig::new("test-build")
        .with_build_type(BuildType::Release);

    // Act
    let res = sys.configure(config);

    // Assert
    assert!(res.is_ok());
    assert_eq!(sys.list_builds().len(), 1);
}
