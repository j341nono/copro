# copro

**copro (Copy with Progress)** is a command-line file backup (copy) tool featuring a simple, rainbow-colored animated progress bar for efficient file operations.

## Installation

### From Source

```bash
git clone https://github.com/j341nono/copro.git
cd copro
cargo build --release
cargo install --path .
```

## Usage

### Basic Usage

```bash
copro [OPTIONS] [SOURCE_POSITIONAL] [DESTINATION_POSITIONAL]
```

### Examples

Copy files from source to destination with progress bar

```bash
copro /path/to/source /path/to/destination
```

Enable verbose output to see detailed copy status for each file

```bash
copro -v /path/to/source /path/to/destination
```

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--source` | `-s` | Source path for the copy operation |
| `--destination` | `-d` | Destination path for the copy operation |
| `--verbose` | `-v` | Show per-file copy success messages |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests on the [GitHub repository](https://github.com/j341nono/copro).
