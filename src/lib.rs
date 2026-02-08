use std::path::Path;

use libsnow::config::configfile::ConfigMode;

lazy_static::lazy_static! {
    pub static ref PKGSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_purple()
        .bold();
    pub static ref VERSIONSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_blue();
    pub static ref ERRORSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_red()
        .bold();
    pub static ref WARNINGSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_yellow()
        .bold();
}

pub mod search;

pub fn is_system_configured() -> bool {
    if let Ok(config) = libsnow::config::configfile::get_config() {
        match config.mode {
            ConfigMode::Toml => config.config_file.is_some(),
            ConfigMode::Nix => config.systemconfig.is_some(),
        }
    } else {
        false
    }
}

pub fn is_home_configured() -> bool {
    if let Ok(config) = libsnow::config::configfile::get_config() {
        match config.mode {
            ConfigMode::Toml => config.config_file.is_some(),
            ConfigMode::Nix => config.homeconfig.is_some(),
        }
    } else {
        false
    }
}

pub fn is_profile_configured() -> bool {
    if let Ok(home) = std::env::var("HOME") {
        Path::new(&format!("{}/.nix-profile/manifest.json", home)).exists()
    } else {
        false
    }
}
