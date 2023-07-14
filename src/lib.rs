use std::process::Command;

pub mod profile;
pub mod search;
pub mod system;

lazy_static::lazy_static! {
    pub static ref PKGSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_purple()
        .bold();
    pub static ref VERSIONSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_blue();
    pub static ref ERRORSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_red()
        .bold();
    pub static ref SYSTEM: String = String::from_utf8_lossy(&Command::new("nix")
        .args(["eval", "--impure", "--raw", "--expr", "builtins.currentSystem"])
        .output()
        .expect("Failed to get current system").stdout).to_string();
}
