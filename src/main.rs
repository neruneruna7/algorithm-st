use std::path::PathBuf;

use algorithm_st::{RunConfig, parse_register_spec, read_input_file, run_file};
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

    /// Initial register values, such as `1=10,2=-3`.
    #[arg(long = "init-register", alias = "init_register", value_parser = parse_register_spec)]
    init_register: Vec<Vec<(usize, algorithm_st::Word)>>,

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
    let result = run_file(
        cli.program_path,
        RunConfig {
            initial_registers: cli.init_register.into_iter().flatten().collect(),
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
