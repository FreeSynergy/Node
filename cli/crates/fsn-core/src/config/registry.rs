// Module registry – scans modules/ directory and loads all module class TOMLs.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::service::ServiceClass;
use crate::error::FsnError;

/// In-memory index of all available module classes.
/// Key = "{type}/{name}" (e.g. "auth/kanidm", "git/forgejo")
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    classes: HashMap<String, ServiceClass>,
    /// Base path of the modules/ directory
    modules_dir: PathBuf,
}

impl ServiceRegistry {
    /// Scan a modules/ directory and load all class TOMLs.
    ///
    /// Two supported layouts:
    ///   Depth 3: `modules/{type}/{name}/{name}.toml`       → key = `{type}/{name}`
    ///   Depth 4: `modules/{type}/{parent}/{name}/{name}.toml` → key = `{type}/{parent}/{name}`
    ///
    /// Depth-4 enables sub-modules nested under a parent module
    /// (e.g. `proxy/zentinel/zentinel-control-plane`).
    pub fn load(modules_dir: &Path) -> Result<Self, FsnError> {
        let mut registry = Self {
            classes: HashMap::new(),
            modules_dir: modules_dir.to_path_buf(),
        };

        for entry in WalkDir::new(modules_dir)
            .min_depth(3)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }

            // File name must match its parent directory (e.g. forgejo/forgejo.toml)
            let file_stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let parent_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or_default();

            if file_stem != parent_name {
                continue;
            }

            // Compute depth relative to modules_dir to pick the right key format
            let depth = path.components().count()
                .saturating_sub(modules_dir.components().count());

            let class_key = if depth == 3 {
                // modules/{type}/{name}/{name}.toml  →  {type}/{name}
                let type_name = path
                    .parent().and_then(|p| p.parent())
                    .and_then(|p| p.file_name()).and_then(|n| n.to_str())
                    .unwrap_or_default();
                format!("{type_name}/{file_stem}")
            } else {
                // modules/{type}/{parent}/{name}/{name}.toml  →  {type}/{parent}/{name}
                let parent_dir = path
                    .parent().and_then(|p| p.file_name()).and_then(|n| n.to_str())
                    .unwrap_or_default();
                let grandparent_dir = path
                    .parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).and_then(|n| n.to_str())
                    .unwrap_or_default();
                let type_name = path
                    .parent().and_then(|p| p.parent()).and_then(|p| p.parent())
                    .and_then(|p| p.file_name()).and_then(|n| n.to_str())
                    .unwrap_or_default();
                format!("{type_name}/{grandparent_dir}/{parent_dir}")
            };

            match Self::load_class(path) {
                Ok(class) => {
                    registry.classes.insert(class_key, class);
                }
                Err(e) => {
                    eprintln!("Warning: skipping {}: {}", path.display(), e);
                }
            }
        }

        Ok(registry)
    }

    fn load_class(path: &Path) -> Result<ServiceClass, FsnError> {
        let content = std::fs::read_to_string(path).map_err(FsnError::Io)?;
        toml::from_str(&content).map_err(|e| FsnError::ConfigParse {
            path: path.display().to_string(),
            source: e,
        })
    }

    /// Look up a module class by its "{type}/{name}" key.
    pub fn get(&self, class_key: &str) -> Option<&ServiceClass> {
        self.classes.get(class_key)
    }

    /// All loaded module classes.
    pub fn all(&self) -> impl Iterator<Item = (&str, &ServiceClass)> {
        self.classes.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn modules_dir(&self) -> &Path {
        &self.modules_dir
    }
}
