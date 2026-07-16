use std::env;
use std::path::PathBuf;

pub const _VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const _APPLICATION_ID: &str = "com.berylllium.harmony";
pub const XDG_DIR_NAME: &str = "harmony";

pub fn config_dir() -> PathBuf {
    portable_config_dir().unwrap_or_else(platform_specific_config_dir)
}

fn portable_config_dir() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let dir = exe.parent()?;

    dir.join(CONFIG_FILE_NAME)
        .is_file()
        .then(|| dir.to_path_buf())
}

fn platform_specific_config_dir() -> PathBuf {
    dirs_next::config_dir()
        .expect("expected valid config dir")
        .join(XDG_DIR_NAME)
}