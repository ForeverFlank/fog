# fog

A functional toy language made as a "fun" "side" project.

## Feature

| Feature          | Status         |
|------------------|----------------|
| Lexer            | ✅ Done        |
| AST Parser       | ✅ Done        |
| Interpreter      | ⏳ In progress |
| Compiler (LLVM?) | 💤 Pending     |

## Why name it fog?

Because

- fog looks like f ∘ g, of which ∘ denotes function composition. This language is a functional language so it fits.
- I had a brain fog making this language.
- You will have a brain fog writing in this language, too.

## Why fog?

Because

- It gives the power of a functionally pure language while having an imperative language-like syntaxes.
- etc.

This is more of a "fun" "side" project, rather than something usable in the real world. Use this if you wish to have some "fun", or to mess around with the language, although I'm happy to know if this language turns out to be actually good.

## Usage

Install [rustc](https://rust-lang.org/tools/install/), then run

```
chmod +x build.sh
./build.sh
./bin/fog <path-to-source-file>
```

### Arguments

- `--print-tokens` – will print the tokens produced by the lexer.
- `--emit-ast` – will emit the AST in PlantUML format.

## Examples

Final state of the project should be able to run something like:

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

## License

MIT