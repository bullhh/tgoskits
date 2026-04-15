use std::{
    env, fs,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DEFAULT_REGISTRY_URL: &str = "https://raw.githubusercontent.com/arceos-hypervisor/axvisor-guest/refs/heads/main/registry/default.toml";
pub const DEFAULT_FALLBACK_REGISTRY_URL: &str = "https://raw.githubusercontent.com/arceos-hypervisor/axvisor-guest/refs/heads/main/registry/v0.0.25.toml";
pub const IMAGE_CONFIG_FILENAME: &str = ".image.toml";
pub const IMAGE_LOCAL_STORAGE_ENV: &str = "AXVISOR_IMAGE_LOCAL_STORAGE";
pub const IMAGE_REGISTRY_ENV: &str = "AXVISOR_IMAGE_REGISTRY";
pub const IMAGE_AUTO_SYNC_ENV: &str = "AXVISOR_IMAGE_AUTO_SYNC";
pub const IMAGE_AUTO_SYNC_THRESHOLD_ENV: &str = "AXVISOR_IMAGE_AUTO_SYNC_THRESHOLD";
const DEFAULT_AUTO_SYNC_THRESHOLD: u64 = 60 * 60 * 24 * 7;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ImageConfig {
    pub local_storage: PathBuf,
    pub registry: String,
    pub auto_sync: bool,
    pub auto_sync_threshold: u64,
}

impl ImageConfig {
    pub fn new_default() -> Self {
        Self {
            local_storage: std::env::temp_dir().join(".axvisor-images"),
            registry: DEFAULT_REGISTRY_URL.to_string(),
            auto_sync: true,
            auto_sync_threshold: DEFAULT_AUTO_SYNC_THRESHOLD,
        }
    }

    pub fn get_config_file_path(base_dir: &Path) -> PathBuf {
        base_dir.join(IMAGE_CONFIG_FILENAME)
    }

    pub fn read_config(base_dir: &Path) -> anyhow::Result<Self> {
        let path = Self::get_config_file_path(base_dir);

        if !path.exists() {
            let config = Self::new_default();
            Self::write_config(base_dir, &config)?;
            return config.with_env_overrides();
        }

        let s = fs::read_to_string(&path)?;
        let config: Self =
            toml::from_str(&s).map_err(|e| anyhow!("Invalid image config file: {e}"))?;
        config.with_env_overrides()
    }

    pub fn write_config(base_dir: &Path, config: &Self) -> anyhow::Result<()> {
        let path = Self::get_config_file_path(base_dir);
        fs::write(path, toml::to_string(config)?)
            .map_err(|e| anyhow!("Failed to write image config file: {e}"))
    }
}

impl ImageConfig {
    fn with_env_overrides(mut self) -> anyhow::Result<Self> {
        if let Some(local_storage) = env_var_non_empty(IMAGE_LOCAL_STORAGE_ENV) {
            self.local_storage = PathBuf::from(local_storage);
        }
        if let Some(registry) = env_var_non_empty(IMAGE_REGISTRY_ENV) {
            self.registry = registry;
        }
        if let Some(auto_sync) = env_var_non_empty(IMAGE_AUTO_SYNC_ENV) {
            self.auto_sync = parse_bool_env(&auto_sync)
                .map_err(|err| anyhow!("{IMAGE_AUTO_SYNC_ENV}: {err}"))?;
        }
        if let Some(threshold) = env_var_non_empty(IMAGE_AUTO_SYNC_THRESHOLD_ENV) {
            self.auto_sync_threshold = threshold.parse::<u64>().map_err(|err| {
                anyhow!("{IMAGE_AUTO_SYNC_THRESHOLD_ENV}: invalid integer `{threshold}`: {err}")
            })?;
        }

        Ok(self)
    }
}

fn env_var_non_empty(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_bool_env(value: &str) -> anyhow::Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(anyhow!(
            "invalid boolean `{value}`; expected one of 1,0,true,false,yes,no,on,off"
        )),
    }
}

pub(crate) fn fallback_registry_url() -> String {
    env::var("AXVISOR_REGISTRY_FALLBACK_URL")
        .unwrap_or_else(|_| DEFAULT_FALLBACK_REGISTRY_URL.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use tempfile::tempdir;

    use super::*;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn read_config_creates_default_when_missing() {
        let dir = tempdir().unwrap();

        let config = ImageConfig::read_config(dir.path()).unwrap();

        assert_eq!(config, ImageConfig::new_default());
        assert!(ImageConfig::get_config_file_path(dir.path()).exists());
    }

    #[test]
    fn read_config_applies_environment_overrides() {
        let _guard = ENV_LOCK.lock().unwrap();
        let dir = tempdir().unwrap();

        unsafe {
            env::set_var(IMAGE_LOCAL_STORAGE_ENV, "/cache/axvisor-images");
            env::set_var(IMAGE_REGISTRY_ENV, "https://example.com/registry.toml");
            env::set_var(IMAGE_AUTO_SYNC_ENV, "false");
            env::set_var(IMAGE_AUTO_SYNC_THRESHOLD_ENV, "42");
        }

        let config = ImageConfig::read_config(dir.path()).unwrap();

        assert_eq!(config.local_storage, PathBuf::from("/cache/axvisor-images"));
        assert_eq!(config.registry, "https://example.com/registry.toml");
        assert!(!config.auto_sync);
        assert_eq!(config.auto_sync_threshold, 42);

        unsafe {
            env::remove_var(IMAGE_LOCAL_STORAGE_ENV);
            env::remove_var(IMAGE_REGISTRY_ENV);
            env::remove_var(IMAGE_AUTO_SYNC_ENV);
            env::remove_var(IMAGE_AUTO_SYNC_THRESHOLD_ENV);
        }
    }
}
