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
