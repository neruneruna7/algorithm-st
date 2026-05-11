use std::io::Write;
use std::path::PathBuf;

use algorithm_st::{RunConfig, Word, read_init_register_file, read_input_file, run_file};
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

    print_initial_state(&initial_registers, &input);

    let result = run_file(
        cli.program_path,
        RunConfig {
            initial_registers,
            input,
            max_steps: cli.max_steps,
        },
    )?;

    print_output(&result.output)?;

    println!("steps: {}", result.steps);
    print_registers("final registers", &result.registers);

    Ok(())
}

fn print_initial_state(initial_registers: &[(usize, Word)], input: &[Word]) {
    let registers = initial_register_snapshot(initial_registers);
    print_registers("initial registers", &registers);

    println!("input:");
    if input.is_empty() {
        println!("  (empty)");
    } else {
        for (index, value) in input.iter().enumerate() {
            println!("  input[{index}] = {value}");
        }
    }
}

fn initial_register_snapshot(initial_registers: &[(usize, Word)]) -> Vec<Word> {
    let register_count = initial_registers
        .iter()
        .map(|(address, _)| *address + 1)
        .max()
        .unwrap_or(1);
    let mut registers = vec![0; register_count];

    for (address, value) in initial_registers {
        registers[*address] = *value;
    }

    registers
}

fn print_output(output: &[Word]) -> std::io::Result<()> {
    println!("WRITE:");
    if output.is_empty() {
        println!("  (empty)");
    } else {
        for value in output {
            println!("  {value}");
        }
    }

    std::io::stdout().flush()
}

fn print_registers(title: &str, registers: &[Word]) {
    println!("{title}:");
    if registers.is_empty() {
        println!("  (none)");
    } else {
        for (address, value) in registers.iter().enumerate() {
            println!("  r{address} = {value}");
        }
    }
}
