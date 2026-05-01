use crate::lexer::{Opcode, Span};

/// RAM命令のオペランドを表す
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Direct(usize),   // 直接アドレッシング
    Indirect(usize), // 間接アドレッシング
    Immediate(i32),  // 即値
}

/// RAM命令を表す
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// ラベル
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub name: String,
    pub span: Span,
}

/// 命令
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionNode {
    pub instruction: Instruction,
    pub span: Span,
}

/// プログラムを構成する上位ノード
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Label(Label),
    Instruction(InstructionNode),
}

/// プログラム全体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}
