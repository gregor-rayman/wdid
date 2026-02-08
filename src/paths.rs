use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::error::{AppError, Result};

pub struct AppPaths {
    pub config_dir: PathBuf,    // ~/.config/wdid
    pub data_dir: PathBuf,      // ~/.local/share/wdid
    pub config_file: PathBuf,   // ~/.config/wdid/config.toml
    pub database_file: PathBuf, // ~/.local/share/wdid/wdid.db
}

impl AppPaths {
    pub fn new() -> Result<Self> {
        let proj = ProjectDirs::from("", "wdid", "wdid")
            .ok_or_else(|| AppError::Path("Could not determine home directory".into()))?;

        let config_dir = proj.config_dir().to_path_buf();
        let data_dir = proj.data_local_dir().to_path_buf();

        Ok(Self {
            config_file: config_dir.join("config.toml"),
            database_file: data_dir.join("wdid.db"),
            config_dir,
            data_dir,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

