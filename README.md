# copro

**copro (short of Copy with Progress)** is a command-line file backup (copy) tool with progress bar for efficient file operations.

Compared to the standard cp command, there may be a 1-5% performance overhead for large files (GB-sized).

## Installation

### From Source

```bash
git clone https://github.com/your-username/copro.git
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
