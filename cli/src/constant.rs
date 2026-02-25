pub const APP_NAME: &str = "Skull";
#[cfg(target_os = "windows")]
pub const ENV_SYSTEM_USER: &str = "USERNAME";
#[cfg(not(target_os = "windows"))]
pub const ENV_SYSTEM_USER: &str = "USER";
pub const ENV_USER: &str = "SKULL_USER";
pub const ENV_PASSWORD: &str = "SKULL_PASSWORD";
pub const ENV_HOST: &str = "SKULL_HOST";
pub const ENDGAME_COOKIE: &str = "endgame";

pub mod path {
    pub fn cache() -> Option<&'static std::path::PathBuf> {
        static PATH: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        PATH.get_or_init(|| dirs::cache_dir().map(|p| p.join(super::APP_NAME)))
            .as_ref()
    }

    pub fn host() -> Option<&'static std::path::PathBuf> {
        static PATH: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
        PATH.get_or_init(|| dirs::data_local_dir().map(|p| p.join(super::APP_NAME).join("host")))
            .as_ref()
    }
}
