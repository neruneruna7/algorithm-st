use std::collections::HashMap;

use thiserror::Error;

use crate::ast::{Instruction, InstructionNode, Item, Label, Program};
use crate::lexer::Span;

pub type InstructionAddress = usize;

/// ラベル定義が指す命令アドレスと、その定義位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelDefinition {
    pub address: InstructionAddress,
    pub span: Span,
}

/// ラベル解決後のプログラム。
///
/// `instructions` は実行対象になる命令列で、`labels` はラベル名から命令アドレスへの表。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProgram {
    pub instructions: Vec<InstructionNode>,
    pub labels: HashMap<String, LabelDefinition>,
}

/// ラベル解決で検出する意味エラー。
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// 同じラベル名が複数回定義されている。
    #[error("duplicate label {name:?} at {new_span}; previous definition at {previous_span}")]
    DuplicateLabel {
        name: String,
        previous_span: Span,
        new_span: Span,
    },

    /// ラベル定義の後に、そのラベルが指す命令が存在しない。
    #[error("label {name:?} does not point to any instruction at {span}")]
    DanglingLabel { name: String, span: Span },

    /// ジャンプ命令または SJ 命令が、未定義のラベルを参照している。
    #[error("undefined label {name:?} at {span}")]
    UndefinedLabel { name: String, span: Span },
}

/// 構文解析済みプログラムのラベルを命令アドレスに解決する。
pub fn resolve(program: Program) -> Result<ResolvedProgram, ResolveError> {
    let mut instructions = Vec::new();
    let mut labels = HashMap::new();
    let mut pending_labels = Vec::new();

    for item in program.items {
        match item {
            // ラベルは「次に現れる命令」を指すため、命令が出るまで保留する。
            Item::Label(label) => pending_labels.push(label),
            Item::Instruction(instruction) => {
                let address = instructions.len();
                define_pending_labels(&mut labels, pending_labels.drain(..), address)?;
                instructions.push(instruction);
            }
        }
    }

    // 入力末尾に残ったラベルは、指す命令が存在しない。
    if let Some(label) = pending_labels.into_iter().next() {
        return Err(ResolveError::DanglingLabel {
            name: label.name,
            span: label.span,
        });
    }

    // ラベル表を作った後で、ジャンプ系命令の参照先が存在するか確認する。
    validate_label_references(&instructions, &labels)?;

    Ok(ResolvedProgram {
        instructions,
        labels,
    })
}

/// 直前までに読んだラベル定義を、現在の命令アドレスへ対応づける。
///
/// すでに同名ラベルが登録されている場合は、再定義として `DuplicateLabel` を返す。
fn define_pending_labels(
    labels: &mut HashMap<String, LabelDefinition>,
    pending_labels: impl Iterator<Item = Label>,
    address: InstructionAddress,
) -> Result<(), ResolveError> {
    // 保留中のラベルは、いま確定した命令アドレスを指す。
    for label in pending_labels {
        // 複数ラベルが同じ命令を指すことは許可するが、同名ラベルの再定義は拒否する。
        if let Some(previous) = labels.get(&label.name) {
            return Err(ResolveError::DuplicateLabel {
                name: label.name,
                previous_span: previous.span.clone(),
                new_span: label.span,
            });
        }

        labels.insert(
            label.name,
            LabelDefinition {
                address,
                span: label.span,
            },
        );
    }

    Ok(())
}

/// 命令列に含まれるラベル参照が、すべて定義済みラベルを指しているか検証する。
///
/// `JUMP`, `JZERO`, `JGTZ`, `SJ` が未定義ラベルを参照していれば `UndefinedLabel` を返す。
fn validate_label_references(
    instructions: &[InstructionNode],
    labels: &HashMap<String, LabelDefinition>,
) -> Result<(), ResolveError> {
    for instruction in instructions {
        match &instruction.instruction {
            // ラベル参照を持つ命令だけを検査する。
            Instruction::Jump { label, .. } | Instruction::Sj { label, .. } => {
                if !labels.contains_key(&label.name) {
                    return Err(ResolveError::UndefinedLabel {
                        name: label.name.clone(),
                        span: label.span.clone(),
                    });
                }
            }
            Instruction::Unary { .. } | Instruction::Halt => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Operand;
    use crate::lexer::Opcode;
    use crate::parser::parse;

    // ラベル定義が直後の命令番号に解決されることを確認する。
    #[test]
    fn resolves_labels_to_instruction_addresses() {
        let program = parse("start: LOAD =1 loop: ADD =1 JUMP loop").unwrap();
        let resolved = resolve(program).unwrap();

        assert_eq!(resolved.instructions.len(), 3);
        assert_eq!(resolved.labels["start"].address, 0);
        assert_eq!(resolved.labels["loop"].address, 1);
    }

    // 連続した複数ラベルは、同じ次命令を指せる。
    #[test]
    fn allows_multiple_labels_for_same_instruction() {
        let program = parse("a: b: LOAD =1").unwrap();
        let resolved = resolve(program).unwrap();

        assert_eq!(resolved.instructions.len(), 1);
        assert_eq!(resolved.labels["a"].address, 0);
        assert_eq!(resolved.labels["b"].address, 0);
    }

    // 同名ラベルの再定義は、ジャンプ先を一意に決められないため拒否する。
    #[test]
    fn rejects_duplicate_labels() {
        let program = parse("loop: LOAD =1 loop: ADD =1").unwrap();
        let error = resolve(program).unwrap_err();

        assert!(matches!(
            error,
            ResolveError::DuplicateLabel { name, .. } if name == "loop"
        ));
    }

    // 入力末尾のラベルは、指す命令が存在しないため拒否する。
    #[test]
    fn rejects_dangling_label_at_end() {
        let program = parse("LOAD =1 end:").unwrap();
        let error = resolve(program).unwrap_err();

        assert!(matches!(
            error,
            ResolveError::DanglingLabel { name, .. } if name == "end"
        ));
    }

    // JUMP の参照先ラベルは、事前または事後に定義されている必要がある。
    #[test]
    fn rejects_undefined_jump_label() {
        let program = parse("JUMP missing").unwrap();
        let error = resolve(program).unwrap_err();

        assert!(matches!(
            error,
            ResolveError::UndefinedLabel { name, .. } if name == "missing"
        ));
    }

    // SJ の第 3 オペランドもラベル参照なので、未定義なら拒否する。
    #[test]
    fn rejects_undefined_sj_label() {
        let program = parse("SJ 0,=1,missing").unwrap();
        let error = resolve(program).unwrap_err();

        assert!(matches!(
            error,
            ResolveError::UndefinedLabel { name, .. } if name == "missing"
        ));
    }

    // resolver は命令を変換せず、命令列とラベル表への分離だけを行う。
    #[test]
    fn keeps_instruction_nodes_unchanged() {
        let program = parse("start: LOAD =1").unwrap();
        let resolved = resolve(program).unwrap();

        assert!(matches!(
            resolved.instructions[0].instruction,
            Instruction::Unary {
                opcode: Opcode::Load,
                operand: Operand::Immediate(1),
            }
        ));
    }
}
