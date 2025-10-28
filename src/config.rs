use parking_lot::RwLock;
use std::sync::Arc;

pub struct SharedConfig {
    pub max_retries: u32,
    pub timeout_ms: u64,
    pub enabled: bool,
}

impl Default for SharedConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_ms: 5000,
            enabled: true,
        }
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<SharedConfig>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(SharedConfig::default())),
        }
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.config.read().enabled
    }

    #[inline]
    pub fn get_max_retries(&self) -> u32 {
        self.config.read().max_retries
    }

    #[inline]
    pub fn get_timeout_ms(&self) -> u64 {
        self.config.read().timeout_ms
    }

    pub fn update_config(&self, max_retries: u32, timeout_ms: u64, enabled: bool) {
        let mut config = self.config.write();
        config.max_retries = max_retries;
        config.timeout_ms = timeout_ms;
        config.enabled = enabled;
    }

    pub fn clone_handle(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
