use crate::lexer::{Opcode, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
/// RAM命令のオペランドを表す
pub enum Operand {
    Direct(usize),   // 直接アドレッシング
    Indirect(usize), // 間接アドレッシング
    Immediate(i32),  // 即値
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// RAM命令を表す
pub enum Instruction {
    Unary {
        opcode: Opcode,
        operand: Operand,
    }, // 通常の命令
    Jump {
        opcode: Opcode,
        label: String,
    }, // ジャンプ命令
    Halt, // 停止命令
    Sj {
        lhs: Operand,
        rhs: Operand,
        label: String,
    }, // Sj命令 特殊なもの
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ラベル
pub struct Label {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// 命令
pub struct InstructionLine {
    pub instruction: Instruction,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// プログラムのアイテムごと，実質的に行ごとに分けたもの
/// ラベルと命令は同じ行に書かないことを前提とする
pub enum ProgramItem {
    Label(Label),
    Instruction(InstructionLine),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// プログラム全体
pub struct Program {
    pub items: Vec<ProgramItem>,
}
