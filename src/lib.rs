//! Build system for SigmaOS.
//!
//! Provides build configuration, dependency resolution, and compilation
//! pipeline management. Simulates build system without actual compilation.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Errors for build operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    BuildNotFound(String),
    DependencyNotFound(String),
    CircularDependency(String),
    BuildFailed(String),
    InvalidConfig(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildNotFound(b) => write!(f, "build not found: {b}"),
            Self::DependencyNotFound(d) => write!(f, "dependency not found: {d}"),
            Self::CircularDependency(c) => write!(f, "circular dependency: {c}"),
            Self::BuildFailed(m) => write!(f, "build failed: {m}"),
            Self::InvalidConfig(m) => write!(f, "invalid config: {m}"),
        }
    }
}

impl std::error::Error for BuildError {}

/// Build configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildConfig {
    pub name: String,
    pub source_dir: PathBuf,
    pub output_dir: PathBuf,
    pub build_type: BuildType,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

impl BuildConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_dir: PathBuf::from("src"),
            output_dir: PathBuf::from("target"),
            build_type: BuildType::Debug,
            dependencies: Vec::new(),
            features: Vec::new(),
        }
    }

    pub fn with_source_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.source_dir = dir.into();
        self
    }

    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    pub fn with_build_type(mut self, build_type: BuildType) -> Self {
        self.build_type = build_type;
        self
    }

    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn validate(&self) -> Result<(), BuildError> {
        if self.name.trim().is_empty() {
            return Err(BuildError::InvalidConfig("name must not be empty".into()));
        }
        if self.dependencies.iter().any(|d| d.trim().is_empty()) {
            return Err(BuildError::InvalidConfig("dependency name must not be empty".into()));
        }
        Ok(())
    }
}

/// Build type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildType {
    Debug,
    Release,
    Profile,
}

impl BuildType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
            Self::Profile => "profile",
        }
    }
}

/// Build status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStatus {
    Pending,
    Building,
    Success,
    Failed,
}

/// Build instance.
#[derive(Debug, Clone, PartialEq)]
pub struct Build {
    pub id: String,
    pub config: BuildConfig,
    pub status: BuildStatus,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub output_path: Option<PathBuf>,
}

/// Build system manager.
#[derive(Debug, Clone, Default)]
pub struct BuildSystem {
    builds: HashMap<String, Build>,
    next_id: u64,
}

impl BuildSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure(&mut self, config: BuildConfig) -> Result<String, BuildError> {
        config.validate()?;
        if self.builds.values().any(|b| b.config.name == config.name) {
            return Err(BuildError::InvalidConfig(format!("duplicate build name: {}", config.name)));
        }

        self.next_id += 1;
        let id = format!("build-{}", self.next_id);
        let build = Build {
            id: id.clone(),
            config: config.clone(),
            status: BuildStatus::Pending,
            started_at: self.next_id,
            finished_at: None,
            output_path: None,
        };
        self.builds.insert(id.clone(), build);
        Ok(id)
    }

    pub fn start_build(&mut self, id: &str) -> Result<(), BuildError> {
        let build = self.builds.get_mut(id)
            .ok_or_else(|| BuildError::BuildNotFound(id.to_string()))?;
        if build.status != BuildStatus::Pending {
            return Err(BuildError::BuildFailed(format!("build not pending: {id}")));
        }
        build.status = BuildStatus::Building;
        Ok(())
    }

    pub fn finish_build(&mut self, id: &str, success: bool) -> Result<(), BuildError> {
        let build = self.builds.get_mut(id)
            .ok_or_else(|| BuildError::BuildNotFound(id.to_string()))?;
        if build.status != BuildStatus::Building {
            return Err(BuildError::BuildFailed(format!("build not building: {id}")));
        }
        build.status = if success { BuildStatus::Success } else { BuildStatus::Failed };
        build.finished_at = Some(self.next_id);
        if success {
            build.output_path = Some(build.config.output_dir.join(&build.config.name));
        }
        Ok(())
    }

    pub fn get_build(&self, id: &str) -> Option<&Build> {
        self.builds.get(id)
    }

    pub fn list_builds(&self) -> Vec<&Build> {
        self.builds.values().collect()
    }

    pub fn remove_build(&mut self, id: &str) -> Result<(), BuildError> {
        self.builds.remove(id)
            .map(|_| ())
            .ok_or_else(|| BuildError::BuildNotFound(id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_build_aaa() {
        // Arrange
        let mut sys = BuildSystem::new();
        let config = BuildConfig::new("test-build")
            .with_build_type(BuildType::Release)
            .with_dependency("lib1");

        // Act
        let id = sys.configure(config);

        // Assert
        assert!(id.is_ok());
        assert_eq!(sys.list_builds().len(), 1);
    }

    #[test]
    fn test_build_lifecycle_aaa() {
        // Arrange
        let mut sys = BuildSystem::new();
        let config = BuildConfig::new("test-build");
        let id = sys.configure(config).unwrap();

        // Act
        sys.start_build(&id).unwrap();
        sys.finish_build(&id, true).unwrap();

        // Assert
        let build = sys.get_build(&id).unwrap();
        assert_eq!(build.status, BuildStatus::Success);
        assert!(build.output_path.is_some());
    }

    #[test]
    fn test_build_not_found_aaa() {
        // Arrange
        let sys = BuildSystem::new();

        // Act
        let res = sys.get_build("nonexistent");

        // Assert
        assert!(res.is_none());
    }
}
