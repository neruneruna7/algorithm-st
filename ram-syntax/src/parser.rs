use thiserror::Error;

use crate::ast::{Instruction, InstructionLine, Label, Operand, Program, ProgramItem};
use crate::lexer::{self, LexError, Opcode, Span, Token, TokenKind};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("expected {expected}, found {found} at {span}")]
    /// 期待したトークンではなかった
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("expected {expected}, found end of input")]
    /// 期待したトークンが見つからず、入力の終わりに達した
    UnexpectedEof { expected: String },

    #[error("instruction is not allowed on label line at {span}")]
    /// ラベル行に命令が続いている
    InstructionOnLabelLine { span: Span },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParseSourceError {
    #[error(transparent)]
    /// 字句解析エラー
    Lex(#[from] LexError),

    #[error(transparent)]
    /// 構文解析エラー
    Parse(#[from] ParseError),
}

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// トークン列全体を構文解析し、意味を持つ行だけを `ProgramItem` として残す。
    pub fn parse(mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();

        while !self.is_at_end() {
            // 空行は RAM プログラムの意味に影響しないため、AST には残さない。
            self.skip_newlines();

            if self.is_at_end() {
                break;
            }

            items.push(self.parse_item()?);
        }

        Ok(Program { items })
    }

    /// 空行ではない 1 行を、ラベル行または命令行として解析する。
    fn parse_item(&mut self) -> Result<ProgramItem, ParseError> {
        if self.is_label_line() {
            self.parse_label_line()
        } else {
            self.parse_instruction_line()
        }
    }

    /// `label:` 形式のラベル行を解析する。
    fn parse_label_line(&mut self) -> Result<ProgramItem, ParseError> {
        let label = self.expect_label_name("label name")?;
        let colon = self.expect_kind(TokenExpectation::Colon)?;
        let span = join_span(&label.span, &colon.span);

        // この言語仕様では、ラベル行と命令行を同じ行に書かない。
        if let Some(token) = self.peek()
            && !matches!(token.kind, TokenKind::Newline)
        {
            return Err(ParseError::InstructionOnLabelLine {
                span: token.span.clone(),
            });
        }

        self.consume_optional_newline();

        Ok(ProgramItem::Label(Label {
            name: label_name(&label),
            span,
        }))
    }

    /// 命令を 1 行ぶん解析し、行全体の span を付ける。
    fn parse_instruction_line(&mut self) -> Result<ProgramItem, ParseError> {
        let start = self.peek().map(|token| token.span.clone()).ok_or_else(|| {
            ParseError::UnexpectedEof {
                expected: "instruction".to_string(),
            }
        })?;
        let instruction = self.parse_instruction()?;
        let end = self.previous_span().unwrap_or_else(|| start.clone());

        self.expect_line_end()?;

        Ok(ProgramItem::Instruction(InstructionLine {
            instruction,
            span: join_span(&start, &end),
        }))
    }

    /// 命令をパースする
    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        let opcode = self.expect_opcode()?;

        // Opcode ごとに、後続に期待する構文が異なる。
        match opcode {
            Opcode::Load
            | Opcode::Store
            | Opcode::Add
            | Opcode::Sub
            | Opcode::Mult
            | Opcode::Div
            | Opcode::Read
            | Opcode::Write => Ok(Instruction::Unary {
                opcode,
                operand: self.parse_operand()?,
            }),
            Opcode::Jump | Opcode::Jzero | Opcode::Jgtz => {
                let label = self.expect_label_name("label name after jump opcode")?;

                Ok(Instruction::Jump {
                    opcode,
                    label: label_name(&label),
                })
            }
            Opcode::Halt => Ok(Instruction::Halt),
            Opcode::Sj => {
                let lhs = self.parse_operand()?;
                self.expect_kind(TokenExpectation::Comma)?;
                let rhs = self.parse_operand()?;
                self.expect_kind(TokenExpectation::Comma)?;
                let label = self.expect_label_name("label name after SJ operands")?;

                Ok(Instruction::Sj {
                    lhs,
                    rhs,
                    label: label_name(&label),
                })
            }
        }
    }

    /// `n`, `*n`, `=n` を、それぞれ直接・間接・即値オペランドに変換する。
    fn parse_operand(&mut self) -> Result<Operand, ParseError> {
        match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Number(_)) => {
                let number = self.expect_number("number")?;
                Ok(Operand::Direct(number as usize))
            }
            Some(TokenKind::Star) => {
                self.advance();
                let number = self.expect_number("number after '*'")?;
                Ok(Operand::Indirect(number as usize))
            }
            Some(TokenKind::Equal) => {
                self.advance();
                let number = self.expect_number("number after '='")?;
                Ok(Operand::Immediate(number))
            }
            Some(_) => Err(self.unexpected_current("operand")),
            None => Err(ParseError::UnexpectedEof {
                expected: "operand".to_string(),
            }),
        }
    }

    /// 現在位置のトークンが opcode であることを確認して読み進める。
    fn expect_opcode(&mut self) -> Result<Opcode, ParseError> {
        let token = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
            expected: "opcode".to_string(),
        })?;

        match token.kind {
            TokenKind::Opcode(opcode) => Ok(opcode),
            _ => Err(unexpected_token("opcode", &token)),
        }
    }

    /// 現在位置のトークンがラベル名であることを確認して読み進める。
    fn expect_label_name(&mut self, expected: &str) -> Result<Token, ParseError> {
        let token = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
            expected: expected.to_string(),
        })?;

        match token.kind {
            TokenKind::LabelName(_) => Ok(token),
            _ => Err(unexpected_token(expected, &token)),
        }
    }

    /// 現在位置のトークンが数値であることを確認して読み進める。
    fn expect_number(&mut self, expected: &str) -> Result<i32, ParseError> {
        let token = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
            expected: expected.to_string(),
        })?;

        match token.kind {
            TokenKind::Number(number) => Ok(number),
            _ => Err(unexpected_token(expected, &token)),
        }
    }

    /// `:` や `,` のような固定記号を期待して読み進める。
    fn expect_kind(&mut self, expectation: TokenExpectation) -> Result<Token, ParseError> {
        let token = self.advance().ok_or_else(|| ParseError::UnexpectedEof {
            expected: expectation.description().to_string(),
        })?;

        if expectation.matches(&token.kind) {
            Ok(token)
        } else {
            Err(unexpected_token(expectation.description(), &token))
        }
    }

    /// 命令の後ろに余分なトークンがないことを確認する。
    fn expect_line_end(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(Token {
                kind: TokenKind::Newline,
                ..
            }) => {
                self.advance();
                Ok(())
            }
            Some(_) => Err(self.unexpected_current("end of line")),
            None => Ok(()),
        }
    }

    /// 行末の改行を 1 つだけ消費する。入力末尾の改行なしも許可する。
    fn consume_optional_newline(&mut self) {
        if matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Newline)
        ) {
            self.advance();
        }
    }

    /// 連続する空行を読み飛ばす。
    fn skip_newlines(&mut self) {
        while matches!(
            self.peek().map(|token| &token.kind),
            Some(TokenKind::Newline)
        ) {
            self.advance();
        }
    }

    /// ラベル行の判定だけは `LabelName Colon` を見るため 2 トークン先読みする。
    fn is_label_line(&self) -> bool {
        matches!(
            (
                self.peek().map(|token| &token.kind),
                self.peek_next().map(|token| &token.kind)
            ),
            (Some(TokenKind::LabelName(_)), Some(TokenKind::Colon))
        )
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_next(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.peek()?.clone();
        self.current += 1;
        Some(token)
    }

    fn previous_span(&self) -> Option<Span> {
        self.current
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map(|token| token.span.clone())
    }

    fn unexpected_current(&self, expected: &str) -> ParseError {
        match self.peek() {
            Some(token) => unexpected_token(expected, token),
            None => ParseError::UnexpectedEof {
                expected: expected.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum TokenExpectation {
    Colon,
    Comma,
}

impl TokenExpectation {
    fn description(self) -> &'static str {
        match self {
            Self::Colon => "':'",
            Self::Comma => "','",
        }
    }

    fn matches(self, kind: &TokenKind) -> bool {
        matches!(
            (self, kind),
            (Self::Colon, TokenKind::Colon) | (Self::Comma, TokenKind::Comma)
        )
    }
}

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Program, ParseError> {
    Parser::new(tokens).parse()
}

pub fn parse(input: &str) -> Result<Program, ParseSourceError> {
    let tokens = lexer::tokenize(input)?;
    Ok(parse_tokens(tokens)?)
}

fn label_name(token: &Token) -> String {
    match &token.kind {
        TokenKind::LabelName(name) => name.clone(),
        _ => unreachable!("token must be a label name"),
    }
}

fn unexpected_token(expected: &str, token: &Token) -> ParseError {
    ParseError::UnexpectedToken {
        expected: expected.to_string(),
        found: token.lexeme.clone(),
        span: token.span.clone(),
    }
}

fn join_span(start: &Span, end: &Span) -> Span {
    Span {
        line: start.line,
        start: start.start,
        end: end.end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_label_and_instruction_lines() {
        let program = parse("start:\nLOAD =10\nJUMP start\nHALT\n").unwrap();

        assert_eq!(
            program,
            Program {
                items: vec![
                    ProgramItem::Label(Label {
                        name: "start".to_string(),
                        span: Span {
                            line: 0,
                            start: 0,
                            end: 6,
                        },
                    }),
                    ProgramItem::Instruction(InstructionLine {
                        instruction: Instruction::Unary {
                            opcode: Opcode::Load,
                            operand: Operand::Immediate(10),
                        },
                        span: Span {
                            line: 1,
                            start: 0,
                            end: 8,
                        },
                    }),
                    ProgramItem::Instruction(InstructionLine {
                        instruction: Instruction::Jump {
                            opcode: Opcode::Jump,
                            label: "start".to_string(),
                        },
                        span: Span {
                            line: 2,
                            start: 0,
                            end: 10,
                        },
                    }),
                    ProgramItem::Instruction(InstructionLine {
                        instruction: Instruction::Halt,
                        span: Span {
                            line: 3,
                            start: 0,
                            end: 4,
                        },
                    }),
                ],
            }
        );
    }

    #[test]
    fn skips_empty_lines() {
        let program = parse("\n\nLOAD 1\n\n").unwrap();

        assert_eq!(
            program.items,
            vec![ProgramItem::Instruction(InstructionLine {
                instruction: Instruction::Unary {
                    opcode: Opcode::Load,
                    operand: Operand::Direct(1),
                },
                span: Span {
                    line: 2,
                    start: 0,
                    end: 6,
                },
            })]
        );
    }

    #[test]
    fn parses_sj_instruction() {
        let program = parse("SJ 0,=2,L1\n").unwrap();

        assert_eq!(
            program.items,
            vec![ProgramItem::Instruction(InstructionLine {
                instruction: Instruction::Sj {
                    lhs: Operand::Direct(0),
                    rhs: Operand::Immediate(2),
                    label: "L1".to_string(),
                },
                span: Span {
                    line: 0,
                    start: 0,
                    end: 10,
                },
            })]
        );
    }

    #[test]
    fn rejects_instruction_on_label_line() {
        let error = parse("loop: LOAD =1\n").unwrap_err();

        assert_eq!(
            error,
            ParseSourceError::Parse(ParseError::InstructionOnLabelLine {
                span: Span {
                    line: 0,
                    start: 6,
                    end: 10,
                },
            })
        );
    }

    const SAMPLE_RAM_DIR: &str = "../../RAM.dir";

    fn sample_ram_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join(SAMPLE_RAM_DIR)
    }

    fn sample_ram_files() -> Vec<std::path::PathBuf> {
        let mut files = std::fs::read_dir(sample_ram_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "ram"))
            .collect::<Vec<_>>();

        files.sort();
        files
    }

    #[test]
    fn parses_all_sample_ram_files() {
        let mut failures = Vec::new();

        for path in sample_ram_files() {
            let input = std::fs::read_to_string(&path).unwrap();

            if let Err(error) = parse(&input) {
                let file_name = path
                    .file_name()
                    .and_then(|file_name| file_name.to_str())
                    .unwrap_or("<unknown>");
                failures.push(format!("{SAMPLE_RAM_DIR}/{file_name}: {error}"));
            }
        }

        assert!(
            failures.is_empty(),
            "failed to parse sample RAM files:\n{}",
            failures.join("\n")
        );
    }
}
