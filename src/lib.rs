pub mod sample;
use std::fmt;
use std::fs;
use std::path::Path;

use ram_syntax::ast::{Instruction, InstructionNode, Operand};
use ram_syntax::lexer::{Opcode, Span};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    Executed(ExecutedInstruction),
    Halted(ExecutedInstruction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedInstruction {
    pub pc: usize,
    pub span: Span,
    pub instruction: String,
}

#[derive(Debug)]
pub enum Error {
    ParseOrResolve(resolver::ResolveSourceError),
    Io(std::io::Error),
    InvalidInputValue {
        value: String,
    },
    InvalidInitRegisterValue {
        line: usize,
        value: String,
    },
    InvalidAddress {
        address: Word,
    },
    ImmediateWrite,
    DivisionByZero,
    InputExhausted,
    StepLimitExceeded {
        max_steps: usize,
    },
    ProgramCounterOutOfBounds {
        pc: usize,
    },
    Runtime {
        pc: usize,
        span: Span,
        instruction: String,
        source: Box<Error>,
    },
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
            Self::Runtime {
                pc,
                span,
                instruction,
                source,
            } => write!(
                f,
                "runtime error at pc {pc}, {span}, while executing `{instruction}`: {source}"
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime { source, .. } => Some(source),
            Self::Io(error) => Some(error),
            Self::ParseOrResolve(error) => Some(error),
            Self::InvalidInputValue { .. }
            | Self::InvalidInitRegisterValue { .. }
            | Self::InvalidAddress { .. }
            | Self::ImmediateWrite
            | Self::DivisionByZero
            | Self::InputExhausted
            | Self::StepLimitExceeded { .. }
            | Self::ProgramCounterOutOfBounds { .. } => None,
        }
    }
}

impl Error {
    fn with_runtime_context(self, pc: usize, span: Span, instruction: &Instruction) -> Self {
        if matches!(self, Self::Runtime { .. }) {
            return self;
        }

        Self::Runtime {
            pc,
            span,
            instruction: format_instruction(instruction),
            source: Box::new(self),
        }
    }
}

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
            if matches!(self.step()?, StepOutcome::Halted(_)) {
                break;
            }
        }

        Ok(RunResult {
            registers: self.registers,
            output: self.output,
            steps: self.steps,
        })
    }

    pub fn step(&mut self) -> Result<StepOutcome, Error> {
        if self.steps >= self.max_steps {
            return Err(Error::StepLimitExceeded {
                max_steps: self.max_steps,
            });
        }

        let Some(node) = self.program.instructions.get(self.pc) else {
            return Err(Error::ProgramCounterOutOfBounds { pc: self.pc });
        };
        let span = node.span.clone();
        let instruction = node.instruction.clone();
        let executed = ExecutedInstruction {
            pc: self.pc,
            span: span.clone(),
            instruction: format_instruction(&instruction),
        };
        self.steps += 1;

        match &instruction {
            Instruction::Unary { opcode, operand } => {
                self.execute_unary(opcode.clone(), operand)
                    .map_err(|error| error.with_runtime_context(self.pc, span, &instruction))?
            }
            Instruction::Jump { opcode, label } => {
                let target = self.program.labels[&label.name].address;
                match opcode {
                    Opcode::Jump => {
                        self.pc = target;
                        return Ok(StepOutcome::Executed(executed));
                    }
                    Opcode::Jzero if self.read_register(0) == 0 => {
                        self.pc = target;
                        return Ok(StepOutcome::Executed(executed));
                    }
                    Opcode::Jgtz if self.read_register(0) > 0 => {
                        self.pc = target;
                        return Ok(StepOutcome::Executed(executed));
                    }
                    Opcode::Jzero | Opcode::Jgtz => {}
                    _ => unreachable!("parser only creates jump instructions for jump opcodes"),
                }
            }
            Instruction::Halt => return Ok(StepOutcome::Halted(executed)),
            Instruction::Sj { lhs, rhs, label } => {
                let rhs_value = self.read_operand(rhs).map_err(|error| {
                    error.with_runtime_context(self.pc, span.clone(), &instruction)
                })?;
                let lhs_value = self.read_operand(lhs).map_err(|error| {
                    error.with_runtime_context(self.pc, span.clone(), &instruction)
                })?;
                let next_value = lhs_value.wrapping_sub(rhs_value);
                self.write_operand(lhs, next_value)
                    .map_err(|error| error.with_runtime_context(self.pc, span, &instruction))?;

                if next_value == 0 {
                    self.pc = self.program.labels[&label.name].address;
                    return Ok(StepOutcome::Executed(executed));
                }
            }
        }

        self.pc += 1;
        Ok(StepOutcome::Executed(executed))
    }

    pub fn current_instruction(&self) -> Option<&InstructionNode> {
        self.program.instructions.get(self.pc)
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn registers(&self) -> &[Word] {
        &self.registers
    }

    pub fn output(&self) -> &[Word] {
        &self.output
    }

    pub fn input_cursor(&self) -> usize {
        self.input_cursor
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

pub fn interpreter_from_source(source: &str, config: RunConfig) -> Result<Interpreter, Error> {
    let program = resolver::resolve_source(source)?;
    Interpreter::new(program, config)
}

pub fn interpreter_from_file(
    path: impl AsRef<Path>,
    config: RunConfig,
) -> Result<Interpreter, Error> {
    let source = fs::read_to_string(path)?;
    interpreter_from_source(&source, config)
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

fn format_instruction(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Unary { opcode, operand } => {
            format!("{} {}", format_opcode(opcode), format_operand(operand))
        }
        Instruction::Jump { opcode, label } => format!("{} {}", format_opcode(opcode), label.name),
        Instruction::Halt => "HALT".to_string(),
        Instruction::Sj { lhs, rhs, label } => format!(
            "SJ {},{},{}",
            format_operand(lhs),
            format_operand(rhs),
            label.name
        ),
    }
}

pub fn format_instruction_for_display(instruction: &Instruction) -> String {
    format_instruction(instruction)
}

fn format_opcode(opcode: &Opcode) -> &'static str {
    match opcode {
        Opcode::Load => "LOAD",
        Opcode::Store => "STORE",
        Opcode::Add => "ADD",
        Opcode::Sub => "SUB",
        Opcode::Mult => "MULT",
        Opcode::Div => "DIV",
        Opcode::Jump => "JUMP",
        Opcode::Jzero => "JZERO",
        Opcode::Jgtz => "JGTZ",
        Opcode::Read => "READ",
        Opcode::Write => "WRITE",
        Opcode::Halt => "HALT",
        Opcode::Sj => "SJ",
    }
}

fn format_operand(operand: &Operand) -> String {
    match operand {
        Operand::Direct(address) => address.to_string(),
        Operand::Indirect(address) => format!("*{address}"),
        Operand::Immediate(value) => format!("={value}"),
    }
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
    fn runtime_errors_include_instruction_context() {
        let error = run_source("LOAD =1\nDIV =0\nHALT\n", RunConfig::default()).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("runtime error at pc 1"));
        assert!(message.contains("line 2, column 1"));
        assert!(message.contains("`DIV =0`"));
        assert!(message.contains("division by zero"));
    }

    #[test]
    fn interpreter_steps_one_instruction_at_a_time() {
        let mut interpreter =
            interpreter_from_source("LOAD =2\nADD =3\nHALT\n", RunConfig::default()).unwrap();

        let first = interpreter.step().unwrap();
        assert!(matches!(first, StepOutcome::Executed(_)));
        assert_eq!(interpreter.registers()[0], 2);
        assert_eq!(interpreter.pc(), 1);

        let second = interpreter.step().unwrap();
        assert!(matches!(second, StepOutcome::Executed(_)));
        assert_eq!(interpreter.registers()[0], 5);
        assert_eq!(interpreter.pc(), 2);

        let third = interpreter.step().unwrap();
        assert!(matches!(third, StepOutcome::Halted(_)));
        assert_eq!(interpreter.steps(), 3);
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

    #[test]
    fn assignment_p1_2_outputs_four_squares_for_initial_register_value() {
        let initial_registers = read_init_register_file("assignment/p1-2.reg").unwrap();
        let n = initial_registers
            .iter()
            .find_map(|(address, value)| (*address == 1).then_some(*value))
            .unwrap();
        let result = run_file(
            "assignment/p1-2.ram",
            RunConfig {
                initial_registers,
                max_steps: 1_000_000,
                ..RunConfig::default()
            },
        )
        .unwrap();

        assert_four_square_output(n, &result.output);
    }

    #[test]
    fn assignment_p1_2_outputs_four_squares_for_random_inputs() {
        const CASES: usize = 300;
        const MAX_N: Word = 1_000;

        let source = std::fs::read_to_string("assignment/p1-2.ram").unwrap();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(43);

        for case_index in 0..CASES {
            let n = random_word_inclusive(&mut rng, 1, MAX_N);
            let result = run_source(
                &source,
                RunConfig {
                    initial_registers: vec![(1, n)],
                    max_steps: 1_000_000,
                    ..RunConfig::default()
                },
            )
            .unwrap_or_else(|error| panic!("case {case_index} failed to run: {error}"));

            assert_eq!(
                result.output.len(),
                4,
                "case {case_index} failed: n = {n}, output = {:?}",
                result.output
            );
            assert_four_square_output(n, &result.output);
        }
    }

    #[test]
    fn assignment_p1_3_computes_factorial_for_initial_register_value() {
        let initial_registers = read_init_register_file("assignment/p1-3.reg").unwrap();
        let n = initial_registers
            .iter()
            .find_map(|(address, value)| (*address == 1).then_some(*value))
            .unwrap();
        let result = run_file(
            "assignment/p1-3.ram",
            RunConfig {
                initial_registers,
                max_steps: 5_000_000,
                ..RunConfig::default()
            },
        )
        .unwrap();

        assert_eq!(result.registers[7], factorial(n));
        assert_eq!(result.output, vec![factorial(n)]);
    }

    #[test]
    fn assignment_p1_3_computes_factorial_for_random_inputs() {
        const CASES: usize = 100;
        const MAX_N: Word = 7;

        let source = std::fs::read_to_string("assignment/p1-3.ram").unwrap();
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(44);

        for case_index in 0..CASES {
            let n = random_word_inclusive(&mut rng, 1, MAX_N);
            let result = run_source(
                &source,
                RunConfig {
                    initial_registers: vec![(1, n)],
                    max_steps: 5_000_000,
                    ..RunConfig::default()
                },
            )
            .unwrap_or_else(|error| panic!("case {case_index} failed to run: {error}"));

            assert_eq!(
                result.registers[7],
                factorial(n),
                "case {case_index} failed"
            );
            assert_eq!(
                result.output,
                vec![factorial(n)],
                "case {case_index} failed"
            );
        }
    }

    #[test]
    fn assignment_p1_3_uses_only_sj_write_and_halt_instructions() {
        let source = std::fs::read_to_string("assignment/p1-3.ram").unwrap();

        for (line_index, line) in source.lines().enumerate() {
            let line = line.split(';').next().unwrap_or_default().trim();
            let line = line
                .split_once(':')
                .map(|(_, instruction)| instruction.trim())
                .unwrap_or(line);
            if line.is_empty() || line.ends_with(':') {
                continue;
            }

            assert!(
                line.starts_with("SJ ") || line.starts_with("WRITE ") || line == "HALT",
                "line {} uses a disallowed instruction: {line}",
                line_index + 1
            );
        }
    }

    fn assert_four_square_output(n: Word, output: &[Word]) {
        assert_eq!(output.len(), 4, "output = {output:?}");
        assert!(
            output.iter().all(|value| *value >= 0),
            "output contains a negative value: {output:?}"
        );
        assert!(
            output.windows(2).all(|pair| pair[0] <= pair[1]),
            "output is not sorted: {output:?}"
        );

        let sum = output.iter().map(|value| value * value).sum::<Word>();
        assert_eq!(sum, n, "output = {output:?}");
    }

    fn factorial(n: Word) -> Word {
        (1..=n).product()
    }

    fn random_usize_inclusive(rng: &mut Xoshiro256PlusPlus, max: usize) -> usize {
        (rng.next_u64() % (max as u64 + 1)) as usize
    }

    fn random_word_inclusive(rng: &mut Xoshiro256PlusPlus, min: Word, max: Word) -> Word {
        let span = i64::from(max) - i64::from(min) + 1;
        min + (rng.next_u64() % span as u64) as Word
    }
}
