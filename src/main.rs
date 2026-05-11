use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use algorithm_st::{
    ExecutedInstruction, Interpreter, RunConfig, StepOutcome, Word, format_instruction_for_display,
    interpreter_from_source, read_init_register_file, read_input_file, run_file,
};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};

#[derive(Debug, Parser)]
#[command(
    name = "algorithm-st",
    about = "Run a RAM program",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// RAM program file to execute.
    program_path: Option<PathBuf>,

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

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the RAM debugger TUI.
    Debug(DebugArgs),
}

#[derive(Debug, Parser)]
struct DebugArgs {
    /// RAM program file to debug.
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
    if let Some(Command::Debug(args)) = cli.command {
        return run_debug(args);
    }

    let Some(program_path) = cli.program_path else {
        return Err("program path is required".into());
    };
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
        program_path,
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

fn run_debug(args: DebugArgs) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(&args.program_path)?;
    let source_lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let input = args
        .input_file
        .map(read_input_file)
        .transpose()?
        .unwrap_or_default();
    let initial_registers = args
        .init_register
        .map(read_init_register_file)
        .transpose()?
        .unwrap_or_default();
    let interpreter = interpreter_from_source(
        &source,
        RunConfig {
            initial_registers,
            input,
            max_steps: args.max_steps,
        },
    )?;

    DebugTerminal::enter()?.run(DebugApp::new(interpreter, source_lines))?;
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

#[derive(Debug)]
struct DebugApp {
    interpreter: Interpreter,
    source_lines: Vec<String>,
    history: Vec<DebugSnapshot>,
    cursor: usize,
    halted: bool,
}

impl DebugApp {
    fn new(interpreter: Interpreter, source_lines: Vec<String>) -> Self {
        let initial = DebugSnapshot::initial(&interpreter);

        Self {
            interpreter,
            source_lines,
            history: vec![initial],
            cursor: 0,
            halted: false,
        }
    }

    fn current(&self) -> &DebugSnapshot {
        &self.history[self.cursor]
    }

    fn previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn next(&mut self) {
        if self.cursor + 1 < self.history.len() {
            self.cursor += 1;
            return;
        }

        if self.halted {
            return;
        }

        match self.interpreter.step() {
            Ok(StepOutcome::Executed(executed)) => {
                self.history.push(DebugSnapshot::after_step(
                    &self.interpreter,
                    executed,
                    "running",
                ));
                self.cursor += 1;
            }
            Ok(StepOutcome::Halted(executed)) => {
                self.halted = true;
                self.history.push(DebugSnapshot::after_step(
                    &self.interpreter,
                    executed,
                    "halted",
                ));
                self.cursor += 1;
            }
            Err(error) => {
                self.halted = true;
                self.history
                    .push(DebugSnapshot::error(&self.interpreter, error.to_string()));
                self.cursor += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct DebugSnapshot {
    step: usize,
    pc: usize,
    line: Option<usize>,
    instruction: String,
    registers: Vec<Word>,
    output: Vec<Word>,
    status: String,
}

impl DebugSnapshot {
    fn initial(interpreter: &Interpreter) -> Self {
        let line = interpreter
            .current_instruction()
            .map(|node| node.span.line + 1);
        let instruction = interpreter
            .current_instruction()
            .map(|node| format_instruction_for_display(&node.instruction))
            .unwrap_or_else(|| "(no instruction)".to_string());

        Self {
            step: interpreter.steps(),
            pc: interpreter.pc(),
            line,
            instruction: format!("next: {instruction}"),
            registers: interpreter.registers().to_vec(),
            output: interpreter.output().to_vec(),
            status: "not started".to_string(),
        }
    }

    fn after_step(
        interpreter: &Interpreter,
        executed: ExecutedInstruction,
        status: impl Into<String>,
    ) -> Self {
        Self {
            step: interpreter.steps(),
            pc: executed.pc,
            line: Some(executed.span.line + 1),
            instruction: executed.instruction,
            registers: interpreter.registers().to_vec(),
            output: interpreter.output().to_vec(),
            status: status.into(),
        }
    }

    fn error(interpreter: &Interpreter, status: String) -> Self {
        let current = interpreter.current_instruction();

        Self {
            step: interpreter.steps(),
            pc: interpreter.pc(),
            line: current.map(|node| node.span.line + 1),
            instruction: current
                .map(|node| format_instruction_for_display(&node.instruction))
                .unwrap_or_else(|| "(no instruction)".to_string()),
            registers: interpreter.registers().to_vec(),
            output: interpreter.output().to_vec(),
            status,
        }
    }
}

struct DebugTerminal {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl DebugTerminal {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    fn run(mut self, mut app: DebugApp) -> io::Result<()> {
        loop {
            self.terminal.draw(|frame| draw_debug(frame, &app))?;

            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) if key.code == KeyCode::Right => app.next(),
                    Event::Key(key) if key.code == KeyCode::Left => app.previous(),
                    Event::Key(key) if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}

impl Drop for DebugTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn draw_debug(frame: &mut ratatui::Frame<'_>, app: &DebugApp) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(rows[1]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[2]);
    let snapshot = app.current();

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "RAM Debugger",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::raw("Left/Right: move  q/Esc: quit"),
        ]),
        Line::from(format!(
            "history {}/{} | step {} | pc {} | line {} | {}",
            app.cursor + 1,
            app.history.len(),
            snapshot.step,
            snapshot.pc,
            snapshot
                .line
                .map(|line| line.to_string())
                .unwrap_or_else(|| "-".to_string()),
            snapshot.status
        )),
    ])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, rows[0]);

    let source_height = columns[0].height.saturating_sub(2) as usize;
    let current_line = snapshot.line.unwrap_or(1);
    let start_line = current_line
        .saturating_sub(source_height.saturating_div(2).max(1))
        .max(1);
    let source_items = app
        .source_lines
        .iter()
        .enumerate()
        .skip(start_line - 1)
        .take(source_height.max(1))
        .map(|(index, line)| {
            let line_number = index + 1;
            let text = format!("{line_number:>4}: {line}");
            if Some(line_number) == snapshot.line {
                ListItem::new(text).style(
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(text)
            }
        })
        .collect::<Vec<_>>();
    let source =
        List::new(source_items).block(Block::default().title("source").borders(Borders::ALL));
    frame.render_widget(source, columns[0]);

    let registers = if snapshot.registers.is_empty() {
        vec![ListItem::new("(none)")]
    } else {
        snapshot
            .registers
            .iter()
            .enumerate()
            .map(|(address, value)| ListItem::new(format!("r{address:<4} = {value}")))
            .collect::<Vec<_>>()
    };
    let registers =
        List::new(registers).block(Block::default().title("registers").borders(Borders::ALL));
    frame.render_widget(registers, columns[1]);

    let instruction = Paragraph::new(format!(
        "executed instruction:\n{}\n\nexecuted line: {}",
        snapshot.instruction,
        snapshot
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "-".to_string())
    ))
    .block(Block::default().title("current step").borders(Borders::ALL))
    .wrap(Wrap { trim: false });
    frame.render_widget(instruction, bottom[0]);

    let output = if snapshot.output.is_empty() {
        "(empty)".to_string()
    } else {
        snapshot
            .output
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    };
    let output = Paragraph::new(output)
        .block(Block::default().title("WRITE").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(output, bottom[1]);
}
