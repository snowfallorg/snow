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
    pub static ref SYSTEM: String = std::fs::read_to_string("/run/current-system/system")
        .unwrap_or_else(|_| "x86_64-linux".to_string());
}
