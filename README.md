# fog

A general purpose, functional toy language made as a "fun" "side" project.

## Examples

Final state of the project should be able to run something like

```fog
fib : Int32 -> Int32
fib n =
    if n == 0, 0
    if n == 1, 1
    else,      fib (n - 1) + fib (n - 2)

num : Int32
num = fib 6

main = num |> toString |> printLine
```

## (Planned) features

- Purely functional
- Eagerly evaluated
- Imperative-like syntaxes
- Strict type system, possibly statically typed with full type erasure

## Progress

| Feature          | Status         |
|------------------|----------------|
| Lexer            | ✅ Done        |
| AST Parser       | ✅ Done        |
| Interpreter      | ⏳ In progress |
| Compiler (LLVM?) | 💤 Pending     |
| Toolings         | 💤 Pending     |

## Why name it fog?

Because

- fog looks like f ∘ g, of which ∘ denotes function composition. This language is a functional language so it fits.
- I had a brain fog making this language.
- You will have a brain fog writing in this language, too.

## Usage

Install [Rust](https://rust-lang.org/tools/install/), clone this repo, then do

```bash
cd fog
cargo build
```

### Arguments

- `--print-tokens` – will print the tokens produced by the lexer.
- `--emit-ast` – will emit the AST in PlantUML format.

## License

MIT
