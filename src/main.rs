use clap::Parser;
use std::process::exit;

use configdiff::cli::CliArgs;
use configdiff::{compare_configs, load_config_file, print_diff_report, DiffOptions};

fn main() {
    let args = CliArgs::parse();

    let source_map = match load_config_file(&args.source) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "Error loading source configuration '{}': {}",
                args.source, e
            );
            exit(2);
        }
    };

    let target_map = match load_config_file(&args.target) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "Error loading target configuration '{}': {}",
                args.target, e
            );
            exit(2);
        }
    };

    let options = DiffOptions {
        redact_secrets: !args.no_redact,
        coerce_types: args.coerce_types,
        ignore_patterns: args.ignore_patterns,
    };

    let report = compare_configs(
        &args.source,
        &source_map,
        &args.target,
        &target_map,
        &options,
    );

    print_diff_report(&report, args.json);

    // Evaluate CI exit conditions
    let should_fail = (args.strict && report.has_diff)
        || (args.fail_on_missing && report.missing_count > 0)
        || (args.fail_on_extra && report.extra_count > 0)
        || (args.fail_on_type && report.type_mismatch_count > 0);

    if should_fail {
        exit(1);
    }
}
