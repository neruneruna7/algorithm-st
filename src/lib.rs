pub mod sample;
use std::fmt;
use std::fs;
use std::path::Path;

use ram_syntax::ast::{Instruction, Operand};
use ram_syntax::lexer::Opcode;
use ram_syntax::resolver::{self, ResolvedProgram};

pub type Word = i32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunConfig {
    pub initial_registers: Vec<(usize, Word)>,
    pub input: Vec<Word>,
    pub max_steps: usize,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            initial_registers: Vec::new(),
            input: Vec::new(),
            max_steps: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub registers: Vec<Word>,
    pub output: Vec<Word>,
    pub steps: usize,
}

#[derive(Debug)]
pub enum Error {
    ParseOrResolve(resolver::ResolveSourceError),
    Io(std::io::Error),
    InvalidInputValue { value: String },
    InvalidInitRegisterValue { line: usize, value: String },
    InvalidAddress { address: Word },
    ImmediateWrite,
    DivisionByZero,
    InputExhausted,
    StepLimitExceeded { max_steps: usize },
    ProgramCounterOutOfBounds { pc: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseOrResolve(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::InvalidInputValue { value } => write!(f, "invalid input value: {value:?}"),
            Self::InvalidInitRegisterValue { line, value } => {
                write!(f, "invalid init_register value at line {line}: {value:?}")
            }
            Self::InvalidAddress { address } => {
                write!(f, "invalid register address: {address}")
            }
            Self::ImmediateWrite => write!(f, "cannot write to an immediate operand"),
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::InputExhausted => write!(f, "input tape is exhausted"),
            Self::StepLimitExceeded { max_steps } => {
                write!(f, "step limit exceeded: {max_steps}")
            }
            Self::ProgramCounterOutOfBounds { pc } => {
                write!(f, "program counter out of bounds: {pc}")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<resolver::ResolveSourceError> for Error {
    fn from(error: resolver::ResolveSourceError) -> Self {
        Self::ParseOrResolve(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone)]
pub struct Interpreter {
    program: ResolvedProgram,
    registers: Vec<Word>,
    input: Vec<Word>,
    input_cursor: usize,
    output: Vec<Word>,
    pc: usize,
    steps: usize,
    max_steps: usize,
}

impl Interpreter {
    pub fn new(program: ResolvedProgram, config: RunConfig) -> Result<Self, Error> {
        let mut registers = Vec::new();

        for (address, value) in config.initial_registers {
            set_register(&mut registers, address, value);
        }

        Ok(Self {
            program,
            registers,
            input: config.input,
            input_cursor: 0,
            output: Vec::new(),
            pc: 0,
            steps: 0,
            max_steps: config.max_steps,
        })
    }

    pub fn run(mut self) -> Result<RunResult, Error> {
        loop {
            if self.steps >= self.max_steps {
                return Err(Error::StepLimitExceeded {
                    max_steps: self.max_steps,
                });
            }

            let Some(node) = self.program.instructions.get(self.pc) else {
                return Err(Error::ProgramCounterOutOfBounds { pc: self.pc });
            };
            let instruction = node.instruction.clone();
            self.steps += 1;

            match instruction {
                Instruction::Unary { opcode, operand } => self.execute_unary(opcode, &operand)?,
                Instruction::Jump { opcode, label } => {
                    let target = self.program.labels[&label.name].address;
                    match opcode {
                        Opcode::Jump => {
                            self.pc = target;
                            continue;
                        }
                        Opcode::Jzero if self.read_register(0) == 0 => {
                            self.pc = target;
                            continue;
                        }
                        Opcode::Jgtz if self.read_register(0) > 0 => {
                            self.pc = target;
                            continue;
                        }
                        Opcode::Jzero | Opcode::Jgtz => {}
                        _ => unreachable!("parser only creates jump instructions for jump opcodes"),
                    }
                }
                Instruction::Halt => break,
                Instruction::Sj { lhs, rhs, label } => {
                    let rhs_value = self.read_operand(&rhs)?;
                    let lhs_value = self.read_operand(&lhs)?;
                    let next_value = lhs_value.wrapping_sub(rhs_value);
                    self.write_operand(&lhs, next_value)?;

                    if next_value == 0 {
                        self.pc = self.program.labels[&label.name].address;
                        continue;
                    }
                }
            }

            self.pc += 1;
        }

        Ok(RunResult {
            registers: self.registers,
            output: self.output,
            steps: self.steps,
        })
    }

    fn execute_unary(&mut self, opcode: Opcode, operand: &Operand) -> Result<(), Error> {
        match opcode {
            Opcode::Load => {
                let value = self.read_operand(operand)?;
                self.write_register(0, value);
            }
            Opcode::Store => {
                let value = self.read_register(0);
                self.write_operand(operand, value)?;
            }
            Opcode::Add => {
                let value = self
                    .read_register(0)
                    .wrapping_add(self.read_operand(operand)?);
                self.write_register(0, value);
            }
            Opcode::Sub => {
                let value = self
                    .read_register(0)
                    .wrapping_sub(self.read_operand(operand)?);
                self.write_register(0, value);
            }
            Opcode::Mult => {
                let value = self
                    .read_register(0)
                    .wrapping_mul(self.read_operand(operand)?);
                self.write_register(0, value);
            }
            Opcode::Div => {
                let divisor = self.read_operand(operand)?;
                if divisor == 0 {
                    return Err(Error::DivisionByZero);
                }
                let value = self.read_register(0).wrapping_div(divisor);
                self.write_register(0, value);
            }
            Opcode::Read => {
                let value = self
                    .input
                    .get(self.input_cursor)
                    .copied()
                    .ok_or(Error::InputExhausted)?;
                self.input_cursor += 1;
                self.write_operand(operand, value)?;
            }
            Opcode::Write => {
                let value = self.read_operand(operand)?;
                self.output.push(value);
            }
            Opcode::Jump | Opcode::Jzero | Opcode::Jgtz | Opcode::Halt | Opcode::Sj => {
                unreachable!("parser separates non-unary instructions")
            }
        }

        Ok(())
    }

    fn read_operand(&self, operand: &Operand) -> Result<Word, Error> {
        match *operand {
            Operand::Immediate(value) => Ok(value),
            Operand::Direct(address) => Ok(self.read_register(address)),
            Operand::Indirect(address) => {
                let indirect_address = self.read_register(address);
                Ok(self.read_register(to_address(indirect_address)?))
            }
        }
    }

    fn write_operand(&mut self, operand: &Operand, value: Word) -> Result<(), Error> {
        match *operand {
            Operand::Immediate(_) => Err(Error::ImmediateWrite),
            Operand::Direct(address) => {
                self.write_register(address, value);
                Ok(())
            }
            Operand::Indirect(address) => {
                let indirect_address = to_address(self.read_register(address))?;
                self.write_register(indirect_address, value);
                Ok(())
            }
        }
    }

    fn read_register(&self, address: usize) -> Word {
        self.registers.get(address).copied().unwrap_or_default()
    }

    fn write_register(&mut self, address: usize, value: Word) {
        set_register(&mut self.registers, address, value);
    }
}

pub fn run_source(source: &str, config: RunConfig) -> Result<RunResult, Error> {
    let program = resolver::resolve_source(source)?;
    Interpreter::new(program, config)?.run()
}

pub fn run_file(path: impl AsRef<Path>, config: RunConfig) -> Result<RunResult, Error> {
    let source = fs::read_to_string(path)?;
    run_source(&source, config)
}

pub fn read_input_file(path: impl AsRef<Path>) -> Result<Vec<Word>, Error> {
    fs::read_to_string(path)?
        .split_whitespace()
        .map(|value| {
            value.parse::<Word>().map_err(|_| Error::InvalidInputValue {
                value: value.to_string(),
            })
        })
        .collect()
}

pub fn read_init_register_file(path: impl AsRef<Path>) -> Result<Vec<(usize, Word)>, Error> {
    fs::read_to_string(path)?
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line_number = index + 1;
            let value =
                line.trim()
                    .parse::<Word>()
                    .map_err(|_| Error::InvalidInitRegisterValue {
                        line: line_number,
                        value: line.to_string(),
                    })?;

            Ok((line_number, value))
        })
        .collect()
}

fn set_register(registers: &mut Vec<Word>, address: usize, value: Word) {
    if registers.len() <= address {
        registers.resize(address + 1, 0);
    }

    registers[address] = value;
}

fn to_address(address: Word) -> Result<usize, Error> {
    usize::try_from(address).map_err(|_| Error::InvalidAddress { address })
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::rngs::Xoshiro256PlusPlus;
    use rand::{Rng, SeedableRng};

    #[test]
    fn runs_arithmetic_and_write() {
        let result = run_source(
            "LOAD =2\nADD =3\nSTORE 1\nWRITE 1\nHALT\n",
            RunConfig::default(),
        )
        .unwrap();

        assert_eq!(result.output, vec![5]);
        assert_eq!(result.registers[1], 5);
    }

    #[test]
    fn reads_from_input_file_values() {
        let result = run_source(
            "READ 1\nREAD 2\nLOAD 1\nADD 2\nWRITE 0\nHALT\n",
            RunConfig {
                input: vec![20, 3],
                ..RunConfig::default()
            },
        )
        .unwrap();

        assert_eq!(result.output, vec![23]);
    }

    #[test]
    fn accepts_initial_registers() {
        let result = run_source(
            "LOAD 4\nADD =1\nWRITE 0\nHALT\n",
            RunConfig {
                initial_registers: vec![(4, 41)],
                ..RunConfig::default()
            },
        )
        .unwrap();

        assert_eq!(result.output, vec![42]);
    }

    #[test]
    fn jumps_by_label() {
        let result = run_source(
            "LOAD =3\nloop: WRITE 0\nSUB =1\nJGTZ loop\nHALT\n",
            RunConfig::default(),
        )
        .unwrap();

        assert_eq!(result.output, vec![3, 2, 1]);
    }

    #[test]
    fn arithmetic_uses_i32_wrapping_semantics() {
        let result = run_source("LOAD 1\nMULT 1\nWRITE 0\nHALT\n", {
            let mut config = RunConfig::default();
            config.initial_registers.push((1, 50_000));
            config
        })
        .unwrap();

        assert_eq!(result.output, vec![50_000_i32.wrapping_mul(50_000)]);
    }

    #[test]
    fn reads_initial_registers_from_file_lines() {
        let path = std::env::temp_dir().join(format!(
            "algorithm-st-init-register-{}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "10\n-3\n0\n").unwrap();

        let registers = read_init_register_file(&path).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(registers, vec![(1, 10), (2, -3), (3, 0)]);
    }

    #[test]
    fn assignment_p1_1_sorts_initial_register_values() {
        let initial_registers = read_init_register_file("assignment/p1-1.reg").unwrap();
        let result = run_file(
            "assignment/p1-1.ram",
            RunConfig {
                initial_registers,
                ..RunConfig::default()
            },
        )
        .unwrap();

        assert_eq!(&result.registers[20..24], &[1, 2, 3, 5]);
    }

    #[test]
    fn assignment_p1_1_matches_rust_sort_with_random_inputs() {
        const CASES: usize = 1_000;
        const MAX_N: usize = 64;
        const MIN_VALUE: Word = -10_000;
        const MAX_VALUE: Word = 10_000;

        let source = std::fs::read_to_string("assignment/p1-1.ram").unwrap();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

        for case_index in 0..CASES {
            let n = random_usize_inclusive(&mut rng, MAX_N);
            let values = (0..n)
                .map(|_| random_word_inclusive(&mut rng, MIN_VALUE, MAX_VALUE))
                .collect::<Vec<_>>();

            let mut expected = values.clone();
            expected.sort();

            let initial_registers = std::iter::once((1, n as Word))
                .chain(
                    values
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(index, value)| (20 + index, value)),
                )
                .collect();
            let result = run_source(
                &source,
                RunConfig {
                    initial_registers,
                    max_steps: 200_000,
                    ..RunConfig::default()
                },
            )
            .unwrap_or_else(|error| panic!("case {case_index} failed to run: {error}"));
            let actual = (0..n)
                .map(|index| {
                    result
                        .registers
                        .get(20 + index)
                        .copied()
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>();

            assert_eq!(
                actual, expected,
                "case {case_index} failed: input = {values:?}"
            );
        }
    }

    fn random_usize_inclusive(rng: &mut Xoshiro256PlusPlus, max: usize) -> usize {
        (rng.next_u64() % (max as u64 + 1)) as usize
    }

    fn random_word_inclusive(rng: &mut Xoshiro256PlusPlus, min: Word, max: Word) -> Word {
        let span = i64::from(max) - i64::from(min) + 1;
        min + (rng.next_u64() % span as u64) as Word
    }
}
