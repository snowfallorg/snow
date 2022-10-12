use ijson::IString;
use serde::{Deserialize, Serialize};

pub mod profile;
pub mod search;
pub mod system;

lazy_static::lazy_static! {
    pub static ref PKGSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_purple();
    pub static ref VERSIONSTYLE: owo_colors::Style = owo_colors::Style::new()
        .bright_blue();
}

#[derive(Debug, Serialize, Deserialize)]
struct NixPkg {
    name: IString,
    pname: IString,
    version: IString,
}
