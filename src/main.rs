use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use miette::{miette, IntoDiagnostic, Result};

use gdtools::config::{load_config, Config};
use gdtools::lint::{run_linter, Diagnostic, Rule, Severity};
use gdtools::rules::all_rules;

#[derive(Parser)]
#[command(name = "gdlint", version, about = "A fast GDScript linter for Godot 4.x")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(global = true, short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Lint GDScript files
    Lint {
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,

        #[arg(short, long, default_value = "text")]
        format: OutputFormat,

        #[arg(short, long)]
        quiet: bool,

        #[arg(short = 'w', long)]
        warnings_as_errors: bool,
    },
    /// Check configuration file
    CheckConfig,
    /// Dump default configuration
    DumpConfig,
    /// List all available rules
    Rules,
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Text,
    Json,
}

fn main() -> ExitCode {
    match run() {
        Ok(has_errors) => {
            if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();

    let config = load_config(cli.config.as_deref()).map_err(|e| miette!(e))?;

    match cli.command.unwrap_or(Command::Lint {
        paths: vec![PathBuf::from(".")],
        format: OutputFormat::Text,
        quiet: false,
        warnings_as_errors: false,
    }) {
        Command::Lint {
            paths,
            format,
            quiet,
            warnings_as_errors,
        } => {
            let has_errors = run_lint(&paths, &config, format, quiet, warnings_as_errors)?;
            Ok(has_errors)
        }
        Command::CheckConfig => {
            println!("Configuration is valid");
            Ok(false)
        }
        Command::DumpConfig => {
            let default = Config::default();
            let toml = toml::to_string_pretty(&default).into_diagnostic()?;
            println!("{}", toml);
            Ok(false)
        }
        Command::Rules => {
            list_rules();
            Ok(false)
        }
    }
}

fn list_rules() {
    let rules = all_rules();

    println!("Available rules:\n");

    let mut by_category: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
    for rule in &rules {
        let meta = rule.meta();
        by_category
            .entry(meta.category.to_string())
            .or_default()
            .push(meta);
    }

    let mut categories: Vec<_> = by_category.keys().cloned().collect();
    categories.sort();

    for category in categories {
        println!("{}:", category.to_uppercase());
        let mut rules = by_category.remove(&category).unwrap();
        rules.sort_by_key(|m| m.id);
        for meta in rules {
            let severity = match meta.default_severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            };
            println!("  {:<30} [{}] {}", meta.id, severity, meta.description);
        }
        println!();
    }
}

fn run_lint(
    paths: &[PathBuf],
    config: &Config,
    format: OutputFormat,
    quiet: bool,
    warnings_as_errors: bool,
) -> Result<bool> {
    let rules = create_rules(config)?;
    let mut all_diagnostics: Vec<Diagnostic> = Vec::new();

    for path in paths {
        if path.is_file() {
            let diagnostics = lint_file(path, &rules, config)?;
            all_diagnostics.extend(diagnostics);
        } else if path.is_dir() {
            let diagnostics = lint_directory(path, &rules, config)?;
            all_diagnostics.extend(diagnostics);
        }
    }

    let has_errors = all_diagnostics.iter().any(|d| {
        d.severity == Severity::Error || (warnings_as_errors && d.severity == Severity::Warning)
    });

    if !quiet {
        output_diagnostics(&all_diagnostics, format);
    }

    Ok(has_errors)
}

fn create_rules(config: &Config) -> Result<Vec<Box<dyn Rule>>> {
    let mut rules = all_rules();

    rules.retain(|r| config.is_rule_enabled(r.meta().id));

    for rule in &mut rules {
        if let Some(rule_config) = config.get_rule_config(rule.meta().id) {
            rule.configure(rule_config).map_err(|e| miette!(e))?;
        }
    }

    Ok(rules)
}

fn lint_file(
    path: &PathBuf,
    rules: &[Box<dyn Rule>],
    config: &Config,
) -> Result<Vec<Diagnostic>> {
    let source = std::fs::read_to_string(path).into_diagnostic()?;
    run_linter(&source, path, rules, config).map_err(|e| miette!("Parse error in {:?}: {}", path, e))
}

fn lint_directory(
    path: &PathBuf,
    rules: &[Box<dyn Rule>],
    config: &Config,
) -> Result<Vec<Diagnostic>> {
    let mut all_diagnostics = Vec::new();

    let walker = WalkBuilder::new(path)
        .standard_filters(true)
        .build();

    for entry in walker {
        let entry = entry.into_diagnostic()?;
        let file_path = entry.path();

        if file_path.extension().map(|e| e == "gd").unwrap_or(false) {
            let should_exclude = config.exclude.iter().any(|pattern| {
                file_path
                    .to_string_lossy()
                    .contains(pattern.trim_matches('*'))
            });

            if !should_exclude {
                match lint_file(&file_path.to_path_buf(), rules, config) {
                    Ok(diagnostics) => all_diagnostics.extend(diagnostics),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
    }

    Ok(all_diagnostics)
}

fn output_diagnostics(diagnostics: &[Diagnostic], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for diag in diagnostics {
                println!("{}", diag);
            }
        }
        OutputFormat::Json => {
            #[derive(serde::Serialize)]
            struct JsonDiagnostic<'a> {
                file: &'a str,
                line: usize,
                column: usize,
                severity: &'a str,
                rule: &'a str,
                message: &'a str,
            }

            let json_diags: Vec<_> = diagnostics
                .iter()
                .map(|d| JsonDiagnostic {
                    file: d.file_path.to_str().unwrap_or(""),
                    line: d.line,
                    column: d.column,
                    severity: match d.severity {
                        Severity::Error => "error",
                        Severity::Warning => "warning",
                        Severity::Info => "info",
                    },
                    rule: &d.rule_id,
                    message: &d.message,
                })
                .collect();

            if let Ok(json) = serde_json::to_string_pretty(&json_diags) {
                println!("{}", json);
            }
        }
    }
}
