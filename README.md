# gdtools

Fast GDScript linter and formatter for Godot 4.x, written in Rust.

## Installation

```bash
cargo install --path .
```

This installs two binaries: `gdlint` and `gdformat`.

## Usage

### Linter (gdlint)

```bash
# Lint current directory
gdlint

# Lint specific files or directories
gdlint lint src/ scripts/player.gd

# Output as JSON
gdlint lint --format json .

# Treat warnings as errors
gdlint lint --warnings-as-errors .

# List available rules
gdlint rules

# Dump default configuration
gdlint dump-config
```

### Formatter (gdformat)

```bash
# Format files in-place
gdformat .

# Check if files need formatting (useful for CI)
gdformat --check .

# Show diff without modifying
gdformat --diff .

# Format stdin to stdout
cat file.gd | gdformat --stdin

# Custom line length
gdformat --line-length 120 .

# Use spaces instead of tabs
gdformat --use-spaces 4 .
```

## Configuration

Create a `gdtools.toml` file in your project root:

```toml
exclude = [".godot/**", "addons/**"]

[rules]
disable = ["trailing-whitespace", "max-line-length"]

[rules.max-line-length]
max = 120

[rules.max-function-args]
max = 8

[format]
line_length = 100
indent_style = "tabs"  # or "spaces"
indent_size = 4        # when using spaces
```

### Inline suppressions

```gdscript
# gdlint:ignore=rule-id
var x = 1  # This line is ignored for rule-id

# gdlint:disable=rule-id
# ... code here is not checked for rule-id
# gdlint:enable=rule-id
```

### Format skip regions

```gdscript
# fmt: off
var x     =    1  # Not formatted
# fmt: on
```

## License

MIT
