use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use ignore::WalkBuilder;
use miette::{miette, IntoDiagnostic, Result};

use gdtools::config::load_config;
use gdtools::format::{compare_ast_with_source, run_formatter, AstCheckResult, FormatOptions, IndentStyle};
use gdtools::parser;

#[derive(Parser)]
#[command(name = "gdformat", version, about = "A fast GDScript formatter for Godot 4.x")]
struct Cli {
    /// Files or directories to format
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,

    /// Check if files are formatted without modifying them
    #[arg(short, long)]
    check: bool,

    /// Show diff without modifying files
    #[arg(short, long)]
    diff: bool,

    /// Read from stdin, write to stdout
    #[arg(long)]
    stdin: bool,

    /// Write formatted output to stdout instead of modifying files
    #[arg(long)]
    stdout: bool,

    /// Maximum line length
    #[arg(short = 'l', long, default_value = "100")]
    line_length: usize,

    /// Use spaces instead of tabs (specify number of spaces)
    #[arg(short = 's', long)]
    use_spaces: Option<usize>,

    /// Path to configuration file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Verify AST equivalence after formatting (catch formatter bugs)
    #[arg(long)]
    check_ast: bool,

    /// Verify formatting is idempotent (formatting twice gives same result)
    #[arg(long)]
    check_idempotent: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(needs_formatting) => {
            if needs_formatting {
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

    // Build format options from CLI or config
    let options = build_options(&cli)?;

    // --check-ast and --check-idempotent imply --check (never write files when verifying)
    let check = cli.check || cli.check_ast || cli.check_idempotent;

    // Handle stdin mode
    if cli.stdin {
        return format_stdin(&options, check, cli.diff, cli.check_ast, cli.check_idempotent);
    }

    // Load config for exclude patterns
    let config = load_config(cli.config.as_deref()).map_err(|e| miette!(e))?;

    let mut any_changes = false;

    for path in &cli.paths {
        if path.is_file() {
            if process_file(path, &options, check, cli.diff, cli.stdout, cli.check_ast, cli.check_idempotent, &config.exclude)? {
                any_changes = true;
            }
        } else if path.is_dir() {
            if process_directory(path, &options, check, cli.diff, cli.stdout, cli.check_ast, cli.check_idempotent, &config.exclude)? {
                any_changes = true;
            }
        }
    }

    Ok(any_changes)
}

fn build_options(cli: &Cli) -> Result<FormatOptions> {
    let indent_style = if let Some(spaces) = cli.use_spaces {
        IndentStyle::Spaces(spaces)
    } else {
        IndentStyle::Tabs
    };

    Ok(FormatOptions {
        indent_style,
        max_line_length: cli.line_length,
        trailing_newline: true,
    })
}

fn format_stdin(options: &FormatOptions, check: bool, diff: bool, check_ast: bool, check_idempotent: bool) -> Result<bool> {
    let mut source = String::new();
    io::stdin()
        .read_to_string(&mut source)
        .into_diagnostic()?;

    let formatted = run_formatter(&source, options).map_err(|e| miette!("{}", e))?;

    if check_ast {
        verify_ast_equivalence("<stdin>", &source, &formatted)?;
    }

    if check_idempotent {
        verify_idempotent("<stdin>", &formatted, options)?;
    }

    if check {
        return Ok(source != formatted);
    }

    if diff {
        print_diff("<stdin>", &source, &formatted);
        return Ok(source != formatted);
    }

    io::stdout()
        .write_all(formatted.as_bytes())
        .into_diagnostic()?;

    Ok(false)
}

fn process_file(
    path: &PathBuf,
    options: &FormatOptions,
    check: bool,
    diff: bool,
    stdout: bool,
    check_ast: bool,
    check_idempotent: bool,
    excludes: &[String],
) -> Result<bool> {
    // Check exclusions
    let path_str = path.to_string_lossy();
    for pattern in excludes {
        if path_str.contains(pattern.trim_matches('*')) {
            return Ok(false);
        }
    }

    let source = std::fs::read_to_string(path).into_diagnostic()?;

    let formatted = match run_formatter(&source, options) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error formatting {:?}: {}", path, e);
            return Ok(false);
        }
    };

    if check_ast {
        verify_ast_equivalence(&path.display().to_string(), &source, &formatted)?;
    }

    if check_idempotent {
        verify_idempotent(&path.display().to_string(), &formatted, options)?;
    }

    let changed = source != formatted;

    if check {
        if changed {
            println!("Would reformat: {}", path.display());
        }
        return Ok(changed);
    }

    if diff {
        if changed {
            print_diff(&path.display().to_string(), &source, &formatted);
        }
        return Ok(changed);
    }

    if stdout {
        io::stdout()
            .write_all(formatted.as_bytes())
            .into_diagnostic()?;
        return Ok(changed);
    }

    // Write formatted output
    if changed {
        std::fs::write(path, &formatted).into_diagnostic()?;
        println!("Formatted: {}", path.display());
    }

    Ok(changed)
}

fn process_directory(
    path: &PathBuf,
    options: &FormatOptions,
    check: bool,
    diff: bool,
    stdout: bool,
    check_ast: bool,
    check_idempotent: bool,
    excludes: &[String],
) -> Result<bool> {
    let mut any_changes = false;

    let walker = WalkBuilder::new(path).standard_filters(true).build();

    for entry in walker {
        let entry = entry.into_diagnostic()?;
        let file_path = entry.path();

        if file_path.extension().map(|e| e == "gd").unwrap_or(false) {
            if process_file(&file_path.to_path_buf(), options, check, diff, stdout, check_ast, check_idempotent, excludes)? {
                any_changes = true;
            }
        }
    }

    Ok(any_changes)
}

fn print_diff(filename: &str, original: &str, formatted: &str) {
    use similar::{ChangeTag, TextDiff};

    println!("--- {}", filename);
    println!("+++ {}", filename);

    let diff = TextDiff::from_lines(original, formatted);

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("...");
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                print!("{}{}", sign, change);
            }
        }
    }
}

fn verify_ast_equivalence(filename: &str, original: &str, formatted: &str) -> Result<()> {
    let original_tree = parser::parse(original).map_err(|e| miette!("Parse error: {}", e))?;
    let formatted_tree = parser::parse(formatted).map_err(|e| miette!("Parse error: {}", e))?;

    match compare_ast_with_source(&original_tree, original, &formatted_tree, formatted) {
        AstCheckResult::Equivalent => Ok(()),
        AstCheckResult::Different { path, difference } => Err(miette!(
            "AST changed after formatting {}!\nPath: {}\nDifference: {}",
            filename,
            path,
            difference
        )),
    }
}

fn verify_idempotent(filename: &str, formatted: &str, options: &FormatOptions) -> Result<()> {
    let formatted_twice = run_formatter(formatted, options).map_err(|e| miette!("{}", e))?;

    if formatted == formatted_twice {
        Ok(())
    } else {
        Err(miette!(
            "Formatting is not idempotent for {}!\nFormatting the output again produces different results.",
            filename
        ))
    }
}
