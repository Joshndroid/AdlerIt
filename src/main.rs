mod app;
mod hash;
mod theme;

use std::{
    io::{self, Read},
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Hex,
    Decimal,
    Both,
}

#[derive(Debug, Parser)]
#[command(version, about = "Adler-32 calculator for desktop and command line")]
struct Args {
    /// Text to hash. Omit all CLI input options to launch the GUI.
    #[arg(short, long, conflicts_with_all = ["file", "stdin"])]
    text: Option<String>,

    /// Hash the raw bytes of a file.
    #[arg(short, long, value_name = "PATH", conflicts_with_all = ["text", "stdin"])]
    file: Option<std::path::PathBuf>,

    /// Read bytes from standard input.
    #[arg(long, conflicts_with_all = ["text", "file"])]
    stdin: bool,

    /// CLI output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Hex)]
    format: OutputFormat,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    if args.text.is_some() || args.file.is_some() || args.stdin {
        return run_cli(args);
    }

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("AdlerIt")
            .with_inner_size([760.0, 520.0])
            .with_min_inner_size([560.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AdlerIt",
        options,
        Box::new(|cc| Ok(Box::new(app::AdlerApp::new(cc)))),
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn run_cli(args: Args) -> Result<()> {
    let bytes = if let Some(text) = args.text {
        text.into_bytes()
    } else if let Some(path) = args.file {
        std::fs::read(&path).with_context(|| format!("could not read {}", path.display()))?
    } else if args.stdin {
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .context("could not read stdin")?;
        bytes
    } else {
        bail!("no input provided")
    };

    let result = hash::adler32(&bytes);
    match args.format {
        OutputFormat::Hex => println!("{}", hash::hex(result)),
        OutputFormat::Decimal => println!("{result}"),
        OutputFormat::Both => println!("{}  {result}", hash::hex(result)),
    }
    Ok(())
}
