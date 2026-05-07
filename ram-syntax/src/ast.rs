use crate::lexer::{Opcode, Span};

/// RAM命令のオペランド
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// 直接アドレッシング
    Direct(usize),
    /// 間接アドレッシング
    Indirect(usize),
    /// 即値アドレッシング
    Immediate(i32),
}

/// RAM命令
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// オペランドを 1 つ取る通常命令。
    Unary { opcode: Opcode, operand: Operand },
    /// ラベル名を 1 つ取るジャンプ命令。
    Jump { opcode: Opcode, label: LabelRef },
    /// 停止命令。
    Halt,
    /// `SJ X,Y,Z`。`X - Y` を `X` に代入し、結果が 0 なら `Z` へジャンプする。
    Sj {
        lhs: Operand,
        rhs: Operand,
        label: LabelRef,
    },
}

/// ラベル参照
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRef {
    /// 参照先のラベル名
    pub name: String,
    /// ラベル参照のソース位置
    pub span: Span,
}

/// ラベル定義
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    /// ラベル名
    pub name: String,
    /// ラベル定義のソース位置
    pub span: Span,
}

/// ソース位置を持つ命令ノード
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionNode {
    /// 命令の意味構造
    pub instruction: Instruction,
    /// 命令ノード全体のソース位置
    pub span: Span,
}

/// 構文解析後の上位ノード
///
/// ラベル解決前なので、ラベル定義と命令は入力順に並ぶ
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Label(Label),
    Instruction(InstructionNode),
}

/// RAMプログラム全体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// ラベル定義または命令ノードの列。
    pub items: Vec<Item>,
}
