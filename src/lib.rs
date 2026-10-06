pub mod cli;
pub mod diff;
pub mod model;
pub mod parser;
pub mod printer;
pub mod redactor;

pub use diff::{compare_configs, DiffOptions};
pub use model::{ConfigValue, DiffEntry, DiffReport, DiffType};
pub use parser::{detect_format, load_config_file, parse_config_content, FileFormat};
pub use printer::print_diff_report;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
