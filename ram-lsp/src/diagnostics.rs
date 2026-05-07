use ram_syntax::diagnostic::{Diagnostic as RamDiagnostic, Severity, analyze_source};
use ram_syntax::lexer::Span;
use tower_lsp::lsp_types::{
    Diagnostic as LspDiagnostic, DiagnosticSeverity, NumberOrString, Position, Range,
};

/// `ram-syntax` の解析結果を LSP の診断リストへ変換する。
pub(crate) fn analyze_to_lsp_diagnostics(text: &str) -> Vec<LspDiagnostic> {
    analyze_source(text)
        .diagnostics
        .into_iter()
        .map(to_lsp_diagnostic)
        .collect()
}

/// 言語非依存の診断を、LSP プロトコルの診断型へ変換する。
fn to_lsp_diagnostic(diagnostic: RamDiagnostic) -> LspDiagnostic {
    LspDiagnostic {
        range: span_to_range(diagnostic.span),
        severity: Some(to_lsp_severity(diagnostic.severity)),
        code: Some(NumberOrString::String("ram-syntax".to_string())),
        code_description: None,
        source: Some("ram-lsp".to_string()),
        message: diagnostic.message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// `ram-syntax` の `Span` は 0 始まりなので、LSP の `Range` にそのまま写せる。
fn span_to_range(span: Span) -> Range {
    Range {
        start: Position::new(span.line as u32, span.start as u32),
        end: Position::new(span.line as u32, span.end as u32),
    }
}

/// 言語非依存の重要度を LSP の重要度へ変換する。
fn to_lsp_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 未定義ラベルの診断が LSP の range/severity に変換されることを確認する。
    #[test]
    fn converts_ram_diagnostics_to_lsp_diagnostics() {
        let diagnostics = analyze_to_lsp_diagnostics("JUMP missing");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].range.start, Position::new(0, 5));
        assert_eq!(diagnostics[0].range.end, Position::new(0, 12));
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
    }
}
