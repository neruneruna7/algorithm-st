module.exports = grammar({
  name: "ram",

  // 空白とタブは構文上の区切りとしてだけ使う。
  // 改行は命令区切りとしても使われ得るため、extras には含めず source_file で扱う。
  extras: ($) => [/[ \t\r]/],

  // ラベル名を word にしておくと、Zed の単語選択や検索が自然に動く。
  word: ($) => $.label_name,

  rules: {
    // RAM の parser 実装では、命令区切りを改行だけに限定していない。
    // そのため Tree-sitter grammar 側も、トップレベルにラベル・命令・コメントを
    // 繰り返し並べられる単純な構造にしている。
    source_file: ($) =>
      repeat(choice($.label_definition, $.instruction, $.comment, "\n")),

    // `loop:` のようなラベル定義。
    // field 名を付けることで highlights.scm から label_name を安定して参照できる。
    label_definition: ($) => seq(field("name", $.label_name), ":"),

    // 命令は opcode と省略可能な operand list で表す。
    // HALT のような 0 オペランド命令を許すため operand_list は optional にする。
    instruction: ($) => seq($.opcode, optional($.operand_list)),

    // SJ などの複数オペランド命令に対応するため、カンマ区切りを許す。
    // 各命令の正確なオペランド数検査は LSP 側の parser/resolver が担当する。
    operand_list: ($) => seq($.operand, repeat(seq(",", $.operand))),

    // Tree-sitter grammar はハイライト用途の軽い構文認識に留める。
    // したがって、どの opcode が label_name を受け取れるかまではここで検査しない。
    operand: ($) =>
      choice($.immediate_operand, $.indirect_operand, $.direct_operand, $.label_name),

    // `=5` はイメディエイトアドレッシング。
    immediate_operand: ($) => seq("=", $.number),

    // `*3` は間接アドレッシング。
    indirect_operand: ($) => seq("*", $.number),

    // `3` は直接アドレッシング。
    direct_operand: ($) => $.number,

    // RAM LSP の lexer/parser が認識する命令セットと合わせる。
    // 命令を追加した場合は、ram-syntax 側の Opcode とこの列挙を同時に更新する。
    opcode: ($) =>
      choice(
        "LOAD",
        "STORE",
        "ADD",
        "SUB",
        "MULT",
        "DIV",
        "JUMP",
        "JZERO",
        "JGTZ",
        "READ",
        "WRITE",
        "HALT",
        "SJ",
      ),

    // lexer.rs の is_label_start / is_label_continue と同じ方針にする。
    // ASCII 英字または '_' で始まり、以降は数字も許す。
    label_name: (_) => /[A-Za-z_][A-Za-z0-9_]*/,

    // 現時点の RAM 構文では負数リテラルを扱わない。
    // 負数を許す場合は lexer/parser とこの grammar の両方を変更する。
    number: (_) => /[0-9]+/,

    // RAM のコメントは `;` から行末まで。
    comment: (_) => /;.*/,
  },
});
