use ram_syntax::ast::Item;
use ram_syntax::parser::parse;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

const OPCODES: &[&str] = &[
    "LOAD", "STORE", "ADD", "SUB", "MULT", "DIV", "JUMP", "JZERO", "JGTZ", "READ", "WRITE", "HALT",
    "SJ",
];

/// RAM ソースから補完候補を生成する。
///
/// 最初の実装では文脈判定を行わず、命令候補と文書内ラベル候補を常に返す。
/// これは、LSP と VS Code 拡張の接続を先に安定させるための単純な方針である。
/// たとえば命令の直後だけラベルを出す、といった文脈依存の制御は parser の
/// エラー回復やカーソル位置の扱いが必要になってから追加する。
pub(crate) fn completion_items(text: Option<&str>) -> Vec<CompletionItem> {
    // 命令名はソースが構文エラーを含んでいても常に提示できる。
    // 編集中のコードは壊れていることが普通なので、固定候補を先に作る。
    let mut items = opcode_completion_items();

    // ラベル候補は現在の文書を parse できた場合だけ追加する。
    // ここでは未完成の文書から無理にラベルを推測せず、誤った候補を出さないことを優先する。
    if let Some(text) = text {
        items.extend(label_completion_items(text));
    }

    items
}

fn opcode_completion_items() -> Vec<CompletionItem> {
    // VS Code では KEYWORD として返すことで、命令名らしい見た目の候補になる。
    OPCODES
        .iter()
        .map(|opcode| CompletionItem {
            label: (*opcode).to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("RAM instruction".to_string()),
            ..CompletionItem::default()
        })
        .collect()
}

fn label_completion_items(text: &str) -> Vec<CompletionItem> {
    // ラベルは AST 上の Label ノードから集める。
    // parser を通すことで、lexer と parser が認識しているラベル定義だけを候補にできる。
    let Ok(program) = parse(text) else {
        return Vec::new();
    };

    program
        .items
        .into_iter()
        .filter_map(|item| match item {
            // ラベル参照補完では、定義位置の末尾 ':' は入力させない。
            // JUMP loop のように参照側では名前だけを書く仕様だからである。
            Item::Label(label) => Some(CompletionItem {
                label: label.name,
                kind: Some(CompletionItemKind::REFERENCE),
                detail: Some("RAM label".to_string()),
                ..CompletionItem::default()
            }),
            Item::Instruction(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_opcode_completion_items() {
        // 文書本文がなくても、固定の命令候補は返す。
        // completion request が文書キャッシュ作成前に来ても最低限の補完を出すためである。
        let items = completion_items(None);

        assert!(items.iter().any(|item| item.label == "LOAD"));
        assert!(items.iter().any(|item| item.label == "JUMP"));
    }

    #[test]
    fn includes_labels_from_current_document() {
        // parse できる文書では、定義済みラベルを補完候補に含める。
        // ここでは loop: が Label ノードになり、補完では loop として提示される。
        let items = completion_items(Some("loop: LOAD =1 JUMP loop"));

        assert!(items.iter().any(|item| item.label == "loop"));
    }

    #[test]
    fn keeps_opcode_items_when_document_has_parse_error() {
        // 編集途中の文書は構文エラーを含むことが多い。
        // その場合でも命令補完まで消えると使いにくいため、固定候補は維持する。
        let items = completion_items(Some("LOAD"));

        assert!(items.iter().any(|item| item.label == "LOAD"));
    }
}
