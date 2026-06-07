# fog

a functional toy language made as a "fun" "side" project.

## status

- [x] lexer
- [x] AST parser
- [ ] interpreter
- [ ] compiler (LLVM?)

## why name it fog?

because

- fog looks like f ∘ g, of which ∘ denotes function composition. this language is a functional language so it fits,
- I had a brain fog making this language, and
- you will have a brain fog writing in this language, too.

## why fog?

because

- it gives the power of a functionally pure language while having an imperative language-like syntaxes.

yup. a single bullet point. this is more of a "fun" "side" project, rather than something production-ready, or something usable in the real world. use this if you wish to have some "fun".

## installation/usage

not at this moment.

## examples

final state of the project should be able to run something like

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

## license

MIT