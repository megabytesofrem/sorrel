# Sorrel Grammar Specification

This document specifies the source-level shape of every node currently represented
by the AST.

- `"token"` is a literal token.
- `name` is a non-terminal.
- `[ item ]` is optional.
- `{ item }` repeats zero or more times.
- `a | b` selects one alternative.
- `# ...` is a grammar comment.

Whitespace, line feeds, `//` comments, and `/* ... */` comments are ignored by
the lexer. `DocComment` (`/+ ... +/`) is retained as a token, but no AST rule
currently associates it with a declaration.

> **Implementation status:** The grammar is AST-complete. The parser currently
> implements primitive, pointer, and array types plus literals, identifiers,
> parenthesized expressions, unary expressions, binary expressions, calls, and
> borrows. The statement, declaration, control-flow, array-literal, field-access,
> range, function-type, and type-variable productions describe existing AST or
> lexer vocabulary that still needs parser support.

## Module and declarations

```ebnf
module:
    { toplevel_stmt }

toplevel_stmt:
    stmt
  | function_decl
  | struct_decl
  | enum_decl

function_decl:
    "def" typed_identifier "(" [ parameters [ "," ] ] ")"
    [ "->" type ] block

parameters:
    typed_identifier { "," typed_identifier }

struct_decl:
    "struct" identifier "{" [ fields [ "," ] ] "}"

enum_decl:
    "enum" identifier "{" [ variants [ "," ] ] "}"

fields:
    typed_identifier { "," typed_identifier }

variants:
    typed_identifier { "," typed_identifier }
```

`FunctionDecl.name`, `StructDecl.name`, and `EnumDecl.name` are represented as
`TypedIdentifier`s in the AST. The grammar therefore permits an annotation on a
declaration name even though such an annotation may later be restricted by
semantic validation.

## Statements and blocks

```ebnf
block:
    "{" { stmt } "}"

stmt:
    expr_stmt
  | let_stmt
  | assign_stmt
  | for_stmt
  | while_stmt

expr_stmt:
    expression

let_stmt:
    "let" typed_identifier "=" expression

assign_stmt:
    assign_target "=" expression

assign_target:
    expression

for_stmt:
    "for" typed_identifier "in" expression block

while_stmt:
    "while" expression block
```

`return` and `break` are lexed keywords but do not yet have matching AST nodes.
Semicolons are lexed and are used by array types; statement terminators are not
yet represented in the AST grammar.

## Expressions

```ebnf
expression:
    assignment

assignment:
    logical_or [ "=" assignment ]

logical_or:
    logical_and { "||" logical_and }

logical_and:
    equality { "&&" equality }

equality:
    comparison { ( "==" | "!=" ) comparison }

comparison:
    term { ( "<" | "<=" | ">" | ">=" ) term }

term:
    factor { ( "+" | "-" ) factor }

factor:
    unary { ( "*" | "/" ) unary }

unary:
    ( "-" | "!" | "*" ) unary
  | borrow
  | postfix

borrow:
    "&" unary region_annotation

region_annotation:
    "'" identifier

postfix:
    primary { call | index | field_access }

call:
    "(" [ arguments [ "," ] ] ")"

arguments:
    expression { "," expression }

index:
    "." "[" expression "]"

field_access:
    "." identifier

primary:
    literal
  | identifier
  | "(" expression ")"
  | array_literal
  | if_expression

array_literal:
    "[" [ arguments [ "," ] ] "]"

if_expression:
    "if" expression block [ "else" ( if_expression | block ) ]
```

### Operator meaning and associativity

```ebnf
binary_operator:
    "+" | "-" | "*" | "/"
  | "+=" | "-=" | "*=" | "/="
  | "==" | "!="
  | "<" | "<=" | ">" | ">="
  | "&&" | "||"

unary_operator:
    "-" | "!" | "*"
```

The AST has a `BinaryOp` node for the compound operators (`+=`, `-=`, `*=`,
`/=`), although they are not presently assigned a precedence by the parser.
The grammar above reserves them as binary operators; their precise syntactic
role should be decided before parser implementation.

The implemented precedence table, from loosest to tightest, is assignment,
logical-or, logical-and, equality, comparison, addition/subtraction,
multiplication/division, unary, and call. Assignment is right-associative;
the other implemented binary levels are left-associative. The AST's `Index`
shape uses `expr.[index]`. `field_access` is lexer and AST-design vocabulary;
there is no dedicated field-access AST node yet.

## Literals and identifiers

```ebnf
literal:
    integer
  | float
  | string
  | character
  | boolean
  | array_literal

integer:
    digit { digit }

float:
    digit { digit } "." digit { digit }

string:
    '"' { string_character | escape_sequence } '"'

character:
    "'" ( character | escape_sequence ) "'"

boolean:
    "true" | "false"

identifier:
    identifier_start { identifier_continue }

identifier_start:
    "a"…"z" | "A"…"Z" | "_"

identifier_continue:
    identifier_start | digit

digit:
    "0"…"9"
```

The integer lexer rule is unsigned syntactically; negativity is represented by
a unary `-` expression. `UInt` exists in the AST but no unsigned literal syntax
is currently defined. String and character escape syntax is accepted lexically;
its decoding semantics are not implemented yet.

## Types

```ebnf
typed_identifier:
    identifier ":" type

type:
    primitive_type
  | pointer_type
  | array_type
  | function_type
  | type_variable

primitive_type:
    "u8" | "u16" | "u32" | "u64"
  | "i8" | "i16" | "i32" | "i64"
  | "f32" | "f64"
  | "usize" | "isize"
  | "bool" | "char" | "str" | "void"

pointer_type:
    "*" type region_annotation

array_type:
    "[" type ";" integer "]"

function_type:
    "(" [ type { "," type } [ "," ] ] ")" "->" type

type_variable:
    identifier
```

`pointer_type` and `array_type` are implemented. Region names are interned, so
two identical names resolve to the same internal `RegionId` during a parse.
`Function` and `Var` are AST type variants without a current parser or lexer
rule; the productions above document a proposed source representation.

## Reserved lexical vocabulary

The lexer additionally reserves the following tokens, which do not all have an
AST production yet:

```ebnf
range_operator:
    ".." | "..."

control_keyword:
    "return" | "break"

delimiter:
    ";" | ":" | "," | "(" | ")" | "[" | "]" | "{" | "}"
```

`%` is also tokenized but has no corresponding operator variant. These tokens
are intentionally omitted from the active expression grammar until their AST
and parser semantics are defined.
