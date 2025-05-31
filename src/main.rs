use clap::Parser;
use console::{style, Term}

fn setup_signal_handler() -> Result<(mpsc::Receiver<()>, Arc<AtomicBool>)> {
    let (tx, rx) = mpsc::channel();
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);
    
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT]).expect("Failed to register signal handler");
        for _ in signals.forever() {
            interrupted_clone.store(true, Ordering::Relaxed);
            let _ = tx.send(());
            break;
        }
    });
    
    Ok((rx, interrupted))
}

fn copy_file_with_temp(source: &Path, destination: &Path) -> std::io::Result<u64> {
    // Create temporary file name
    let temp_dest = destination.with_extension(
        format!("{}.tmp", 
            destination.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("tmp")
        )
    );
    
    // Copy to temporary file first
    let bytes_copied = fs::copy(source, &temp_dest)?;
    
    // Rename temporary file to final destination (atomic operation)
    fs::rename(&temp_dest, destination)?;
    
    Ok(bytes_copied)
};
use dialoguer::Input;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    sync::mpsc,
};
use anyhow::Result;
use signal_hook::{consts::SIGINT, iterator::Signals};

/// File copy tool with dynamic terminal animation
#[derive(Parser)]
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

    /// skip temporary file protection for maximum speed
    #[arg(long)]
    fast_mode: bool,

    /// reduce animation update frequency for better performance
    #[arg(long)]
    low_animation: bool,
}

struct AnimatedProgress {
    term: Term,
    current: Arc<Mutex<usize>>,
    total: usize,
    start_time: Instant,
    animation_chars: Vec<&'static str>,
    wave_chars: Vec<&'static str>,
    colors: Vec<console::Color>,
    should_stop: Arc<AtomicBool>,
    animation_handle: Option<thread::JoinHandle<()>>,
}

impl AnimatedProgress {
    fn new(total: usize) -> Self {
        Self {
            term: Term::stdout(),
            current: Arc::new(Mutex::new(0)),
            total,
            start_time: Instant::now(),
            animation_chars: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            wave_chars: vec!["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"],
            colors: vec![
                console::Color::Red,
                console::Color::Yellow,
                console::Color::Green,
                console::Color::Cyan,
                console::Color::Blue,
                console::Color::Magenta,
            ],
            should_stop: Arc::new(AtomicBool::new(false)),
            animation_handle: None,
        }
    }

    fn start_animation(&mut self, low_animation: bool) {
        let current = Arc::clone(&self.current);
        let total = self.total;
        let term = self.term.clone();
        let animation_chars = self.animation_chars.clone();
        let wave_chars = self.wave_chars.clone();
        let colors = self.colors.clone();
        let start_time = self.start_time;
        let should_stop = Arc::clone(&self.should_stop);

        let handle = thread::spawn(move || {
            let mut frame = 0;
            while !should_stop.load(Ordering::Relaxed) {
                let current_count = *current.lock().unwrap();
                if current_count >= total {
                    break;
                }

                let elapsed = start_time.elapsed().as_secs_f32();
                let spinner = animation_chars[frame % animation_chars.len()];
                
                // Create dynamic wave effect
                let mut wave_bar = String::new();
                for i in 0..20 {
                    let wave_offset = (elapsed * 3.0 + i as f32 * 0.3).sin();
                    let wave_index = ((wave_offset + 1.0) * 3.5) as usize % wave_chars.len();
                    let color_index = (frame / 2 + i) % colors.len();
                    wave_bar.push_str(&format!("{}", style(wave_chars[wave_index]).fg(colors[color_index])));
                }

                // Progress percentage with rainbow effect
                let progress = if total > 0 { (current_count as f32 / total as f32 * 100.0) as u8 } else { 0 };
                let progress_color = match progress {
                    0..=20 => console::Color::Red,
                    21..=40 => console::Color::Yellow,
                    41..=60 => console::Color::Green,
                    61..=80 => console::Color::Cyan,
                    81..=100 => console::Color::Magenta,
                    _ => console::Color::White,
                };

                // Create pulsing effect for file counter
                let pulse_intensity = (elapsed * 4.0).sin().abs();
                let file_counter_style = if pulse_intensity > 0.7 {
                    style(format!("{}/{}", current_count, total)).bold().fg(console::Color::White)
                } else {
                    style(format!("{}/{}", current_count, total)).fg(console::Color::Cyan)
                };

                // Animated brackets
                let bracket_char = if (frame / 5) % 2 == 0 { "◤" } else { "◢" };
                let bracket_style = style(bracket_char).fg(colors[frame % colors.len()]);

                // Build the complete animation line
                let animation_line = format!(
                    "\r{} {} {} {} {}% {} Copying files... {} {}",
                    bracket_style,
                    style(spinner).fg(console::Color::Green).bold(),
                    wave_bar,
                    bracket_style,
                    style(progress).fg(progress_color).bold(),
                    file_counter_style,
                    style("✨").fg(console::Color::Yellow),
                    if frame % 20 < 10 { "🚀" } else { "⚡" }
                );

                let _ = term.write_str(&animation_line);
                let _ = term.flush();
                
                // Configurable animation speed
                let sleep_duration = if low_animation { 200 } else { 100 };
                thread::sleep(Duration::from_millis(sleep_duration));
                frame += 1;
            }
        });

        self.animation_handle = Some(handle);
    }

    fn increment(&self) {
        let mut current = self.current.lock().unwrap();
        *current += 1;
    }

    fn stop_animation(&mut self) {
        self.should_stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.animation_handle.take() {
            let _ = handle.join();
        }
    }

    fn finish(&mut self) {
        self.stop_animation();
        
        let current_count = *self.current.lock().unwrap();
        let elapsed = self.start_time.elapsed();
        
        // Clear the animation line
        let _ = self.term.write_str("\r");
        let _ = self.term.clear_line();
        
        // Show completion message with celebration effects
        let completion_line = format!(
            "🎉 {} {} files copied in {:.2}s! {} 🎊\n",
            style("SUCCESS!").green().bold(),
            style(current_count).cyan().bold(),
            elapsed.as_secs_f32(),
            style("COMPLETE").magenta().bold()
        );
        
        let _ = self.term.write_str(&completion_line);
        let _ = self.term.flush();
    }

    fn interrupted(&mut self) {
        self.stop_animation();
        
        let current_count = *self.current.lock().unwrap();
        let elapsed = self.start_time.elapsed();
        
        // Clear the animation line
        let _ = self.term.write_str("\r");
        let _ = self.term.clear_line();
        
        // Show interruption message
        let interruption_line = format!(
            "\n🛑 {} Operation interrupted after {:.2}s\n📊 Progress: {}/{} files copied\n⚠️  {} Some files may be partially copied\n",
            style("INTERRUPTED!").red().bold(),
            elapsed.as_secs_f32(),
            style(current_count).yellow().bold(),
            style(self.total).yellow().bold(),
            style("WARNING:").yellow().bold()
        );
        
        let _ = self.term.write_str(&interruption_line);
        let _ = self.term.flush();
    }
}

fn setup_signal_handler() -> Result<(mpsc::Receiver<()>, Arc<AtomicBool>)> {
    let (tx, rx) = mpsc::channel();
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = Arc::clone(&interrupted);
    
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT]).expect("Failed to register signal handler");
        for _ in signals.forever() {
            interrupted_clone.store(true, Ordering::Relaxed);
            let _ = tx.send(());
            break;
        }
    });
    
    Ok((rx, interrupted))
}


fn copy_file_with_temp(source: &Path, destination: &Path) -> std::io::Result<u64> {
    // Create temporary file name
    let temp_dest = destination.with_extension(
        format!("{}.tmp", 
            destination.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("tmp")
        )
    );
    
    // Copy to temporary file first
    let bytes_copied = fs::copy(source, &temp_dest)?;
    
    // Rename temporary file to final destination (atomic operation)
    fs::rename(&temp_dest, destination)?;
    
    Ok(bytes_copied)
}

fn collect_files(path: &Path) -> Result<Vec<PathBuf>> {
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

    // Set up signal handler for graceful shutdown
    let (interrupt_rx, interrupted) = setup_signal_handler()?;

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
    let file_count = files.len();
    
    println!("🚀 {} Starting copy operation...", style("INITIALIZING").cyan().bold());
    println!("📁 Files to copy: {}", style(file_count).yellow().bold());
    println!("💾 Total size: {} bytes", style(total_bytes).green().bold());
    println!("💡 Press Ctrl+C to safely stop the operation");
    println!();

    let mut progress = AnimatedProgress::new(file_count);
    progress.start_animation(cli.low_animation);

    // Small delay to let animation start
    thread::sleep(Duration::from_millis(200));

    for file in files {
        // Check for interruption before each file
        if interrupted.load(Ordering::Relaxed) {
            progress.interrupted();
            return Ok(());
        }

        // Check for interruption signal (non-blocking)
        if interrupt_rx.try_recv().is_ok() {
            progress.interrupted();
            return Ok(());
        }

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
    
        // Use safe copy with temporary file (unless fast mode)
        match if cli.fast_mode {
            fs::copy(&file, &dest_path)
        } else {
            copy_file_with_temp(&file, &dest_path)
        } {
            Ok(_) => {
                progress.increment();
                if cli.verbose {
                    println!("\n{} {}", 
                        style("✅ Success:").green().bold(),
                        style(file.display()).white()
                    );
                }
            }
            Err(e) => {
                println!("\n{} {} ({})", 
                    style("❌ Failed:").red().bold(),
                    style(file.display()).white(),
                    style(e).red()
                );
                
                // Clean up any partial temporary files
                let temp_dest = dest_path.with_extension(
                    format!("{}.tmp", 
                        dest_path.extension()
                            .and_then(|s| s.to_str())
                            .unwrap_or("tmp")
                    )
                );
                let _ = fs::remove_file(&temp_dest);
            }
        }
        
        // Add slight delay between files to show animation better
        thread::sleep(Duration::from_millis(50));
    }

    progress.finish();
    Ok(())
}