use std::path::PathBuf;

use algorithm_st::{RunConfig, read_init_register_file, read_input_file, run_file};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "algorithm-st",
    about = "Run a RAM program",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    /// RAM program file to execute.
    program_path: PathBuf,

    /// Input tape file. Values are read as whitespace-separated i32 integers.
    #[arg(long = "input-file", alias = "input_file")]
    input_file: Option<PathBuf>,

    /// Initial register file. Line 1 initializes r1, line 2 initializes r2, and so on.
    #[arg(long = "init-register", alias = "init_register")]
    init_register: Option<PathBuf>,

    /// Maximum number of instructions to execute before stopping.
    #[arg(long = "max-steps", default_value_t = 1_000_000)]
    max_steps: usize,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let input = cli
        .input_file
        .map(read_input_file)
        .transpose()?
        .unwrap_or_default();
    let initial_registers = cli
        .init_register
        .map(read_init_register_file)
        .transpose()?
        .unwrap_or_default();
    let result = run_file(
        cli.program_path,
        RunConfig {
            initial_registers,
            input,
            max_steps: cli.max_steps,
        },
    )?;

    for value in result.output {
        println!("{value}");
    }
    eprintln!("steps: {}", result.steps);

    Ok(())
}
