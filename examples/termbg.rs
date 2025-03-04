use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let num_args = args.len();

    if num_args > 1 {
        match args[1].as_str() {
            "--version" | "-V" => {
                println!("termbg {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--debug" | "-d" => {
                CombinedLogger::init(vec![TermLogger::new(
                    LevelFilter::Debug,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                )])
                .unwrap();
            }
            _ => {
                eprintln!("Usage: {} [--debug/-d] [--version/-V]", args[0]);
                std::process::exit(1);
            }
        }
    }

    let timeout = std::time::Duration::from_millis(100);

    println!("Check terminal background color");
    let term = termbg::terminal();
    let latency = termbg::latency(std::time::Duration::from_millis(1000));
    let rgb = termbg::rgb(timeout);
    let theme = termbg::theme(timeout);

    println!("  Term : {:?}", term);

    match latency {
        Ok(latency) => {
            println!("  Latency: {:?}", latency);
        }
        Err(e) => {
            println!("  Latency: detection failed {:?}", e);
        }
    }

    match rgb {
        Ok(rgb) => {
            println!("  Color: R={:x}, G={:x}, B={:x}", rgb.r, rgb.g, rgb.b);
        }
        Err(e) => {
            println!("  Color: detection failed {:?}", e);
        }
    }

    match theme {
        Ok(theme) => {
            println!("  Theme: {:?}", theme);
        }
        Err(e) => {
            println!("  Theme: detection failed {:?}", e);
        }
    }
}
