use clap::Parser;
use console::style;
use dialoguer::Input;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    path::{Path, PathBuf},
};
use anyhow::Result;

/// BackUP CLI Tool
#[derive(Parser)]
#[command(name = "bak")]
#[command(version = "0.1")]
#[command(about = "File backup (copy) tool with progress bar", long_about = None)]
struct Cli {
    /// source path
    #[arg(short, long)]
    source: Option<PathBuf>,

    /// target path
    #[arg(short, long)]
    destination: Option<PathBuf>,

    /// source path as positional argument
    source_positional: Option<PathBuf>,

    /// destination path as positional argument
    destination_positional: Option<PathBuf>,

    /// show per-file copy success messages
    #[arg(short, long)]
    verbose: bool,
}

fn collect_files(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                files.extend(collect_files(&path)?);
            }
        }
    }
    Ok(files)
}

fn total_size(files: &[PathBuf]) -> u64 {
    files.iter()
        .filter_map(|f| fs::metadata(f).ok())
        .map(|m| m.len())
        .sum()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = cli.source.or(cli.source_positional)
        .unwrap_or_else(|| {
            PathBuf::from(
                Input::<String>::new()
                    .with_prompt("Enter source path")
                    .interact_text()
                    .unwrap()
            )
        });

    let destination = cli.destination.or(cli.destination_positional)
        .unwrap_or_else(|| {
            PathBuf::from(
                Input::<String>::new()
                    .with_prompt("Enter destination path")
                    .interact_text()
                    .unwrap()
            )
        });

    let files = collect_files(&source)?;
    let total_bytes = total_size(&files);
    let bar = ProgressBar::new(total_bytes);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {bytes}/{total_bytes} bytes Copying...")
            .unwrap(),
    );

    println!("{} ファイルをコピーします（合計サイズ: {} bytes）", files.len(), total_bytes);

    for file in files {
        let rel_path = file.strip_prefix(&source).unwrap_or(&file);
        let dest_path = if source.is_file() {
            if destination.is_dir() {
                destination.join(
                    source.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
                )
            } else {
                destination.clone()
            }
        } else {
            destination.join(rel_path)
        };
    
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
    
        match fs::copy(&file, &dest_path) {
            Ok(copied_bytes) => {
                bar.inc(copied_bytes);
                if cli.verbose {
                    bar.println(format!("{}", style(format!("✅ 成功: {}", file.display())).green()));
                }
            }
            Err(e) => {
                bar.println(format!("{}", style(format!("❌ 失敗: {} ({})", file.display(), e)).red()));
            }
        }
    }
    bar.finish_with_message("完了しました！");
    Ok(())
}