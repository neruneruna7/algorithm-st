; Labels are definition sites such as `loop:`.
; ラベル参照ではなく、定義位置だけを @label として強調する。
(label_definition
  name: (label_name) @label)

; RAM instruction names.
; 命令セットは grammar.js の opcode に列挙しているため、ここではそのノードを
; keyword として扱えば全命令が同じ規則でハイライトされる。
(opcode) @keyword

; Addressing-mode prefixes and separators.
; `=5`, `*3`, `SJ X,Y,Z` のような記法を読みやすくするため、接頭辞と区切りを
; punctuation として扱う。
[
  ":"
  ","
  "="
  "*"
] @punctuation.delimiter

; RAM のレジスタ番号・即値は現時点ではすべて同じ number ノードにしている。
(number) @number

; lexer と同様に、`;` から行末までをコメントとして扱う。
(comment) @comment
