use crate::lexer::{LexError, Lexer, Span};
use crate::parser::{ParseError, Parser};
use crate::resolver::{ResolveError, ResolvedProgram, Resolver};

/// LSP の DiagnosticSeverity に対応させるための診断重要度。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// LSP に渡す直前の、言語非依存な診断情報。
///
/// LSP 固有型へはサーバ層で変換する。`Span` は RAM ソース上の位置を表す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub severity: Severity,
}

/// 解析結果。
///
/// 正常時は `resolved` にラベル解決済みプログラムが入り、`diagnostics` は空になる。
/// 異常時は `resolved` が `None` になり、診断が 1 件以上入る。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub resolved: Option<ResolvedProgram>,
    pub diagnostics: Vec<Diagnostic>,
}

/// RAM ソースを LSP 診断へ変換しやすい形まで解析する。
///
/// 現段階では、字句解析・構文解析・ラベル解決の最初のエラーを 1 件返す。
pub fn analyze_source(input: &str) -> Analysis {
    // LSP では編集中の不完全なソースを受け取るため、各段階で失敗を診断に変換する。
    let tokens = match Lexer::new(input).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => return failed_analysis(lex_error_to_diagnostic(error)),
    };

    let program = match Parser::new(tokens).parse() {
        Ok(program) => program,
        Err(error) => return failed_analysis(parse_error_to_diagnostic(error, input)),
    };

    let resolved = match Resolver::new().resolve(program) {
        Ok(resolved) => resolved,
        Err(error) => return failed_analysis(resolve_error_to_diagnostic(error)),
    };

    // すべての段階が成功した場合だけ、後続機能が利用できる解決済みプログラムを返す。
    Analysis {
        resolved: Some(resolved),
        diagnostics: Vec::new(),
    }
}

/// エラー 1 件で解析を打ち切る場合の `Analysis` を作る。
fn failed_analysis(diagnostic: Diagnostic) -> Analysis {
    Analysis {
        resolved: None,
        diagnostics: vec![diagnostic],
    }
}

/// 字句解析エラーを診断へ変換する。
///
/// 字句解析エラーはすでに `Span` を持っているため、その位置をそのまま使う。
fn lex_error_to_diagnostic(error: LexError) -> Diagnostic {
    let span = match &error {
        LexError::InvalidChar { span, .. } | LexError::InvalidNumber { span, .. } => span.clone(),
    };

    Diagnostic {
        message: error.to_string(),
        span,
        severity: Severity::Error,
    }
}

/// 構文解析エラーを診断へ変換する。
///
/// `UnexpectedEof` はエラー自身に位置を持たないため、入力末尾の `Span` を補う。
fn parse_error_to_diagnostic(error: ParseError, input: &str) -> Diagnostic {
    let span = match &error {
        ParseError::UnexpectedToken { span, .. } => span.clone(),
        ParseError::UnexpectedEof { .. } => eof_span(input),
    };

    Diagnostic {
        message: error.to_string(),
        span,
        severity: Severity::Error,
    }
}

/// ラベル解決エラーを診断へ変換する。
///
/// 重複ラベルでは新しい定義位置を、未定義参照では参照位置を診断位置にする。
fn resolve_error_to_diagnostic(error: ResolveError) -> Diagnostic {
    let span = match &error {
        ResolveError::DuplicateLabel { new_span, .. } => new_span.clone(),
        ResolveError::DanglingLabel { span, .. } | ResolveError::UndefinedLabel { span, .. } => {
            span.clone()
        }
    };

    Diagnostic {
        message: error.to_string(),
        span,
        severity: Severity::Error,
    }
}

/// 入力末尾を 0 幅の `Span` として表す。
///
/// LSP の range へ変換するとき、EOF に対する構文エラーを最後のカーソル位置へ出すために使う。
fn eof_span(input: &str) -> Span {
    let mut line = 0;
    let mut column = 0;

    for ch in input.chars() {
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
    }

    Span {
        line,
        start: column,
        end: column,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 正常系では診断が空で、ラベル解決済みプログラムが返る。
    #[test]
    fn analyzes_valid_source() {
        let analysis = analyze_source("start: LOAD =1 JUMP start");

        assert!(analysis.diagnostics.is_empty());
        assert_eq!(analysis.resolved.unwrap().labels["start"].address, 0);
    }

    // 字句解析で止まった場合は、その不正文字の位置を診断にする。
    #[test]
    fn reports_lex_error_as_diagnostic() {
        let analysis = analyze_source("LOAD @");

        assert!(analysis.resolved.is_none());
        assert_eq!(analysis.diagnostics.len(), 1);
        assert_eq!(
            analysis.diagnostics[0].span,
            Span {
                line: 0,
                start: 5,
                end: 6,
            }
        );
    }

    // 入力終端で構文要素が足りない場合は、EOF 位置を診断にする。
    #[test]
    fn reports_parse_error_as_diagnostic() {
        let analysis = analyze_source("LOAD");

        assert!(analysis.resolved.is_none());
        assert_eq!(analysis.diagnostics.len(), 1);
        assert_eq!(
            analysis.diagnostics[0].span,
            Span {
                line: 0,
                start: 4,
                end: 4,
            }
        );
    }

    // ラベル解決で止まった場合は、未定義ラベル参照の位置を診断にする。
    #[test]
    fn reports_resolve_error_as_diagnostic() {
        let analysis = analyze_source("JUMP missing");

        assert!(analysis.resolved.is_none());
        assert_eq!(analysis.diagnostics.len(), 1);
        assert_eq!(
            analysis.diagnostics[0].span,
            Span {
                line: 0,
                start: 5,
                end: 12,
            }
        );
    }
}
