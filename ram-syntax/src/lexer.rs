use thiserror::Error;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,  // 出現した行番号
    pub start: usize, // 行内での開始位置
    pub end: usize,   // 行内での終了位置
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line + 1, self.start + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opcode {
    Load,  // r0にデータコピー
    Store, // r0の内容を別の場所にコピー
    Add,   // r0 <- r0 + 別のメモリ
    Sub,   // r0 <- r0 - 別のメモリ
    Mult,  // r0 <- r0 * 別のメモリ
    Div,   // r0 <- r0 / 別のメモリ
    Jump,  // 与えられたラベルにジャンプ
    Jzero, // r0=0ならば 与えられたラベルにジャンプ
    Jgtz,  // r0>0ならば 与えられたラベルにジャンプ
    Read,  // テープからメモリに読み込む
    Write, // メモリからテープに書き込む
    Halt,  // 停止する
    Sj,    // オペランドX,Y,Z  X - Y をXに代入して，Xが0であればラベルZへJUMPする
}

// #[derive(Debug, Clone)]
// pub enum Oprand {
//     Direct(usize),   // 直接アドレッシング
//     Indirect(usize), // 間接アドレッシング
//     Imediate(i32),   // 即値
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Opcode(Opcode),
    LabelName(String),
    Number(i32),
    Equal,
    Star,
    Comma,
    Colon,
    Newline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String, // トークンに対応する文字列
    pub span: Span,     // トークンの位置情報
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LexError {
    #[error("invalid character {ch:?} at {span}")]
    InvalidChar { ch: char, span: Span },

    #[error("invalid number {lexeme:?} at {span}")]
    InvalidNumber { lexeme: String, span: Span },
}

#[derive(Debug, Clone)]
pub struct Lexer {
    input: Vec<char>,
    current: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            current: 0,
            line: 0,
            column: 0,
        }
    }

    /// 入力文字列をトークンのベクタに変換する。
    /// 不正な文字があればエラーを返す。
    pub fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    tokens.push(self.single_char_token(TokenKind::Newline));
                    self.line += 1;
                    self.column = 0;
                }
                ';' => self.skip_comment(),
                '=' => tokens.push(self.single_char_token(TokenKind::Equal)),
                '*' => tokens.push(self.single_char_token(TokenKind::Star)),
                ',' => tokens.push(self.single_char_token(TokenKind::Comma)),
                ':' => tokens.push(self.single_char_token(TokenKind::Colon)),
                '0'..='9' => tokens.push(self.number()?),
                ch if is_label_start(ch) => tokens.push(self.word()),
                ch => {
                    let span = self.span_at_current(1);
                    return Err(LexError::InvalidChar { ch, span });
                }
            }
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.current).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.current += 1;
        self.column += 1;
        Some(ch)
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let line = self.line;
        let start = self.column;
        let lexeme = self
            .advance()
            .expect("single-char token must exist")
            .to_string();

        Token {
            kind,
            lexeme,
            span: Span {
                line,
                start,
                end: start + 1,
            },
        }
    }

    fn number(&mut self) -> Result<Token, LexError> {
        let line = self.line;
        let start = self.column;
        let lexeme = self.take_while(|ch| ch.is_ascii_digit());
        let number = lexeme.parse::<i32>().map_err(|_| LexError::InvalidNumber {
            lexeme: lexeme.clone(),
            span: Span {
                line,
                start,
                end: self.column,
            },
        })?;

        Ok(Token {
            kind: TokenKind::Number(number),
            lexeme,
            span: Span {
                line,
                start,
                end: self.column,
            },
        })
    }

    fn word(&mut self) -> Token {
        let line = self.line;
        let start = self.column;
        let lexeme = self.take_while(is_label_continue);
        let kind = Opcode::from_keyword(&lexeme)
            .map(TokenKind::Opcode)
            .unwrap_or_else(|| TokenKind::LabelName(lexeme.clone()));

        Token {
            kind,
            lexeme,
            span: Span {
                line,
                start,
                end: self.column,
            },
        }
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut lexeme = String::new();

        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }

            lexeme.push(ch);
            self.advance();
        }

        lexeme
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }

            self.advance();
        }
    }

    fn span_at_current(&self, width: usize) -> Span {
        Span {
            line: self.line,
            start: self.column,
            end: self.column + width,
        }
    }
}

impl Opcode {
    fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword.to_ascii_uppercase().as_str() {
            "LOAD" => Some(Self::Load),
            "STORE" => Some(Self::Store),
            "ADD" => Some(Self::Add),
            "SUB" => Some(Self::Sub),
            "MULT" => Some(Self::Mult),
            "DIV" => Some(Self::Div),
            "JUMP" => Some(Self::Jump),
            "JZERO" => Some(Self::Jzero),
            "JGTZ" => Some(Self::Jgtz),
            "READ" => Some(Self::Read),
            "WRITE" => Some(Self::Write),
            "HALT" => Some(Self::Halt),
            "SJ" => Some(Self::Sj),
            _ => None,
        }
    }
}

/// 入力文字列を RAM のトークン列に変換する。
///
/// 空白とコメントは読み飛ばし、改行は `TokenKind::Newline` として残す。
///
/// ```
/// use ram_syntax::lexer::{tokenize, Opcode, TokenKind};
///
/// let input = "LOAD =10\nSTORE *3\nL1:\tSJ 0,=2,L2 ; comment\n";
/// let tokens = tokenize(input).unwrap();
///
/// assert_eq!(
///     tokens.into_iter().map(|token| token.kind).collect::<Vec<_>>(),
///     vec![
///         TokenKind::Opcode(Opcode::Load),
///         TokenKind::Equal,
///         TokenKind::Number(10),
///         TokenKind::Newline,
///         TokenKind::Opcode(Opcode::Store),
///         TokenKind::Star,
///         TokenKind::Number(3),
///         TokenKind::Newline,
///         TokenKind::LabelName("L1".to_string()),
///         TokenKind::Colon,
///         TokenKind::Opcode(Opcode::Sj),
///         TokenKind::Number(0),
///         TokenKind::Comma,
///         TokenKind::Equal,
///         TokenKind::Number(2),
///         TokenKind::Comma,
///         TokenKind::LabelName("L2".to_string()),
///         TokenKind::Newline,
///     ]
/// );
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(input).tokenize()
}

// ラベル名の開始文字として有効な文字か
fn is_label_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

// ラベル名の続きの文字として有効な文字か
fn is_label_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_basic_instructions_and_operands() {
        let tokens = tokenize("LOAD =10\nSTORE *3\n").unwrap();

        assert_eq!(
            tokens
                .into_iter()
                .map(|token| token.kind)
                .collect::<Vec<_>>(),
            vec![
                TokenKind::Opcode(Opcode::Load),
                TokenKind::Equal,
                TokenKind::Number(10),
                TokenKind::Newline,
                TokenKind::Opcode(Opcode::Store),
                TokenKind::Star,
                TokenKind::Number(3),
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn tokenizes_labels_and_compact_sj_operands() {
        let tokens = tokenize("L1:\tSJ 0,=2,L2 ; comment\n").unwrap();

        assert_eq!(
            tokens
                .into_iter()
                .map(|token| token.kind)
                .collect::<Vec<_>>(),
            vec![
                TokenKind::LabelName("L1".to_string()),
                TokenKind::Colon,
                TokenKind::Opcode(Opcode::Sj),
                TokenKind::Number(0),
                TokenKind::Comma,
                TokenKind::Equal,
                TokenKind::Number(2),
                TokenKind::Comma,
                TokenKind::LabelName("L2".to_string()),
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn rejects_invalid_characters() {
        let error = tokenize("LOAD @10").unwrap_err();

        assert_eq!(
            error,
            LexError::InvalidChar {
                ch: '@',
                span: Span {
                    line: 0,
                    start: 5,
                    end: 6,
                },
            }
        );
    }
}
