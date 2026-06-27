use anyhow::Result;
use chrono::Local;
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "CLIPomodoro", about = "A CLI Pomodoro timer")]
struct Cli {
    /// Work duration in minutes
    #[arg(short, long, default_value = "25")]
    work: u64,

    /// Break duration in minutes
    #[arg(short, long, default_value = "5")]
    break_min: u64,

    /// Number of cycles
    #[arg(short, long, default_value = "4")]
    cycles: u32,

    /// Start paused (press Enter to resume)
    #[arg(short = 'p', long)]
    start_paused: bool,
}

fn clear_line() {
    print!("\r\x1b[2K");
}

fn format_time(remaining: u64) -> String {
    let mins = remaining / 60;
    let secs = remaining % 60;
    format!("{:02}:{:02}", mins, secs)
}

fn draw_progress_bar(elapsed: u64, total: u64, width: usize) -> String {
    let filled = if total > 0 {
        ((elapsed as f64 / total as f64) * width as f64) as usize
    } else {
        0
    };
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    let bar: String = std::iter::repeat('█')
        .take(filled)
        .chain(std::iter::repeat('░').take(empty))
        .collect();
    bar
}

fn ring_bell() {
    print!("\x07");
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

fn wait_for_resume(paused: &Arc<AtomicBool>) {
    println!("  ⏸  PAUSED — Press Ctrl+C to resume");
    // The ctrlc handler will flip the flag; we just busy-wait
    while paused.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(100));
    }
}

fn run_timer(
    label: &str,
    duration_mins: u64,
    paused: &Arc<AtomicBool>,
    stop: &Arc<AtomicBool>,
    pause_flag: &Arc<AtomicBool>,
) -> bool {
    let total_secs = duration_mins * 60;
    let bar_width = 30;

    for elapsed in 0..=total_secs {
        if stop.load(Ordering::SeqCst) {
            return false;
        }

        if pause_flag.load(Ordering::SeqCst) {
            wait_for_resume(pause_flag);
        }

        let remaining = total_secs - elapsed;
        let now = Local::now().format("%H:%M:%S").to_string();
        let bar = draw_progress_bar(elapsed, total_secs, bar_width);
        let time_str = format_time(remaining);

        clear_line();
        print!(
            "\r  {label} |{bar}| {time_str} [{now}]"
        );
        std::io::Write::flush(&mut std::io::stdout()).ok();

        if elapsed < total_secs {
            sleep(Duration::from_secs(1));
        }
    }

    println!();
    ring_bell();
    true
}

fn print_phase(cycle: u32, total_cycles: u32, phase: &str, duration_mins: u64) {
    println!(
        "\n━━━ Cycle {cycle}/{total_cycles} — {phase} ({duration_mins} min) ━━━\n"
    );
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(cli.start_paused));
    let pause_flag = Arc::clone(&paused);
    let stop_flag = Arc::clone(&running);

    // ctrlc handler: first press toggles pause, second press quits
    ctrlc::set_handler(move || {
        static mut PAUSE_TOGGLE: bool = false;
        unsafe {
            if PAUSE_TOGGLE {
                stop_flag.store(true, Ordering::SeqCst);
            } else {
                pause_flag.store(true, Ordering::SeqCst);
                PAUSE_TOGGLE = true;
            }
        }
    })?;

    println!("╔══════════════════════════╗");
    println!("║     🍅  CLIPomodoro     ║");
    println!("╚══════════════════════════╝");
    println!();
    println!("  Work: {} min | Break: {} min | Cycles: {}", cli.work, cli.break_min, cli.cycles);
    println!("  Press Ctrl+C to pause/resume. Press Ctrl+C twice to quit.");
    if cli.start_paused {
        println!("  Starting paused — press Ctrl+C to begin.");
    }

    if cli.start_paused {
        wait_for_resume(&paused);
        paused.store(false, Ordering::SeqCst);
    }

    for cycle in 1..=cli.cycles {
        // Work phase
        print_phase(cycle, cli.cycles, "FOCUS", cli.work);
        if !run_timer("🍅 FOCUS", cli.work, &paused, &running, &paused) {
            break;
        }

        if cycle < cli.cycles {
            // Break phase (short break)
            print_phase(cycle, cli.cycles, "BREAK", cli.break_min);
            if !run_timer("☕ BREAK", cli.break_min, &paused, &running, &paused) {
                break;
            }
        } else {
            // Long break after last cycle
            let long_break = cli.break_min * 2;
            print_phase(cycle, cli.cycles, "LONG BREAK", long_break);
            if !run_timer("🎉 LONG BREAK", long_break, &paused, &running, &paused) {
                break;
            }
        }
    }

    println!("\n  ✅ Session complete! Great focus! 🎉\n");
    Ok(())
}
