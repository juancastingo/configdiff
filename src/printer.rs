use crate::model::{DiffEntry, DiffReport, DiffType};
use colored::*;

pub fn print_diff_report(report: &DiffReport, as_json: bool) {
    if as_json {
        let json_str = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
        println!("{}", json_str);
        return;
    }

    println!();
    println!("{}", "═".repeat(70).cyan());
    println!(
        " {} {} {} {}",
        "🔍 ConfigDiff:".bold().white(),
        report.source_path.bold().yellow(),
        "→".dimmed(),
        report.target_path.bold().cyan()
    );
    println!("{}", "═".repeat(70).cyan());

    // Overview Stats
    println!(
        " Keys: Source: {}, Target: {} | Identical: {}",
        report.total_source_keys.to_string().yellow(),
        report.total_target_keys.to_string().cyan(),
        report.identical_count.to_string().green()
    );

    let summary_parts = [
        format!("Missing: {}", report.missing_count.to_string().red()),
        format!("Extra: {}", report.extra_count.to_string().green()),
        format!(
            "Type Mismatches: {}",
            report.type_mismatch_count.to_string().yellow()
        ),
        format!(
            "Value Mismatches: {}",
            report.value_mismatch_count.to_string().blue()
        ),
    ];
    println!(" Summary: {}", summary_parts.join(" | "));
    println!("{}", "─".repeat(70).cyan());

    if !report.has_diff {
        println!(
            " {}",
            "✓ Configurations are semantically identical!"
                .bold()
                .green()
        );
        println!("{}", "═".repeat(70).cyan());
        println!();
        return;
    }

    println!("{}", " DIFFERENCES:".bold());

    for entry in &report.diffs {
        print_diff_entry(entry);
    }

    println!("{}", "═".repeat(70).cyan());
    println!();
}

fn print_diff_entry(entry: &DiffEntry) {
    let secret_badge = if entry.is_secret {
        format!(" {}", "[SECRET]".bold().red())
    } else {
        "".to_string()
    };

    match entry.diff_type {
        DiffType::Missing => {
            let symbol = "-".bold().red();
            let val = entry.source_val.as_deref().unwrap_or("none");
            let t = entry.source_type.as_deref().unwrap_or("");
            let desc = format!("(missing in target, was {}: {})", t, val).dimmed();
            println!(
                "  {} {:<32} {}{}",
                symbol,
                entry.path.bold().red(),
                desc,
                secret_badge
            );
        }
        DiffType::Extra => {
            let symbol = "+".bold().green();
            let val = entry.target_val.as_deref().unwrap_or("none");
            let t = entry.target_type.as_deref().unwrap_or("");
            let desc = format!("(extra in target, {}: {})", t, val).dimmed();
            println!(
                "  {} {:<32} {}{}",
                symbol,
                entry.path.bold().green(),
                desc,
                secret_badge
            );
        }
        DiffType::TypeMismatch => {
            let symbol = "~".bold().yellow();
            let s_t = entry.source_type.as_deref().unwrap_or("unknown");
            let t_t = entry.target_type.as_deref().unwrap_or("unknown");
            let s_v = entry.source_val.as_deref().unwrap_or("");
            let t_v = entry.target_val.as_deref().unwrap_or("");
            let desc = format!("{}({}) -> {}({})", s_t, s_v, t_t, t_v).yellow();
            println!(
                "  {} {:<32} {}{}",
                symbol,
                entry.path.bold().yellow(),
                desc,
                secret_badge
            );
        }
        DiffType::ValueMismatch => {
            let symbol = "≠".bold().cyan();
            let s_v = entry.source_val.as_deref().unwrap_or("");
            let t_v = entry.target_val.as_deref().unwrap_or("");
            let desc = format!("{} -> {}", s_v.dimmed(), t_v.cyan());
            println!(
                "  {} {:<32} {}{}",
                symbol,
                entry.path.bold().white(),
                desc,
                secret_badge
            );
        }
    }
}
