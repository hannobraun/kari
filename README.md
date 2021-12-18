# Kari

## Introduction

``` kari
"std" import

"The answer to life, the universe, and everything is: " print
1 [ 2 3 7 ] [ * ] fold println
```

Kari is a programming language that is

- **embeddable**: Designed to be embedded into a host application.
- **interpreted**: The language is parsed and executed by an interpreter.
- **dynamic**: Types are checked at runtime.
- **strong**: Type conversions are always explicit.
- **stack-based**: There are no variables. All data lives on an implicit stack.
- **postfix**: An operation is preceded by its arguments

Kari is a general-purpose language, but by itself, it has very limited abilities to interact with the outside world. It is designed to be embedded into a host application, that would extend it to provide whatever domain-specific capabilities required.

Kari is written in [Rust], and designed to be embedded into Rust applications.

Example use cases would be:

- To computationally define geometry in a CAD application.
- As a scripting language for a game engine.
- A turing-complete configuration language for a complex application.

And whatever else you can come up with.


## Current Status

Kari is usable, but still very raw and immature. It is lacking a lot of basic features and the ways it can be extended are currently limited. Don't use Kari for anything serious, unless you're perfectly comfortable with working on its interpreter.

That said, if you like the concepts presented here and are are willing to tinker, Kari could be a good place to start.


## Usage

You can run Kari programs like this (requires [`just`]):

```
git clone https://github.com/kari-lang/kari.git
just run hello # or run any other example from `kr/examples`
just test # run all tests from `kr/tests`
```

To run your own programs, you should be able `cargo install` the Kari interpreter and use it to run your programs wherever they are located. This is currently not documented.

The [minimal-host] repository demonstrates how to embed Kari into a host application.

[minimal-host]: https://github.com/kari-lang/minimal-host.git


## Tutorial

Kari is a concatenative, stack-based language, which means there are no variables, and all data lives on an implicit stack. Consider the following Kari program.

``` kari
1
```

This program consists of a single word, `1`, which will push the value `1` to the stack. Integers are actually implemented as special syntax, but from a conceptual point of view, you can view `1` as a function that takes no input and pushes the value `1` on the stack.

Words in Kari are delimited by whitespace. A program with multiple words just executes the functions those words refer to in series.

``` kari
1 2
```

This program consists of two words, which will push the values `1` and `2` to the stack. Kari doesn't care about which whitespace you use to delimit words, so the following program is completely equivalent.

``` kari
1
2
```

So far we've only looked at functions that push words to the stack. But of course, functions can also pop values from the stack.

``` kari
1 2 +
```

This program will push the values `1` and `2` to the stack. Then the `+` function will pop those two numbers from the stack and push `3`, their sum. So the only value left on the stack at the end of the program is `3`.

We can make this program more self-explanatory by adding a comment.

``` kari
1 2 + # sums up `1` and `2`, leaving `3` on the stack
```

Comments are started with `#` and last until the next line break.

There are other values in Kari besides numbers.

``` kari
true # a boolean
1 # an integer
2.0 # a float
"a string"
:a_symbol # symbols are much like strings, except they don't allow whitespace
[ 1 2 3 ] # a list of numbers
[ "a" "list" "of" "strings" ]
```

There are more functions we can call. Some are builtins, that are defined in the global namespace, others are defined in Kari's standard library, and have to be imported. The following program will load the standard library and import its functions into the local namespace.

``` kari
"std" import
```

Once we did that we can use those functions defined in `std`, alongside the builtin functions.

``` kari
"std" import

"Hello, world!" println # prints "Hello, world!" and a line break

1 2 = # compares `1` and `2`; leaves `false` on the stack, as they're not equal

true assert # will do nothing, but `false assert` would have failed the program
```

Since functions all share the implicit stack, we can chain different functions together to create more complex operations.

``` kari
1 2 + # add `1` and `2`, leaving `3` on the stack
3 =   # push another `3`, compare those `3`s using the equals operator
# `3` and `3` are equal, so the previous operation left `true` on the stack
assert # assert that the top value on the stack is `true`; consumes the value
```

Or shorter:

``` kari
1 2 + 3 = assert # test the `+` function
```

There's a lot more to learn about the functions Kari provides, but let's close this tutorial with an important concept: How to define your own functions.

We've already seen all the concepts needed to understand function definition:

- We've seen lists, and an anomymous function is just a list of words.
- We've seen symbols, which can be used to define a function name.

The last piece we're missing is the builtin function `define`, which takes an anonymous function and a symbol, to define a new named function.

``` kari
[ 2 + ] :add_two define

1 add_two 3 = assert
```


## Reference

Currently there's no reference documentation that explains all language features, builtin functions and the standard library. Please refer to the following material:

- The [Kari code] in the repository. Please note that a lot of the examples are outdated and don't work, but the [tests] are up-to-date.
- The [builtins.rs] file, which defines all built-in functions.
- The [Kari standard library].


## License

Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.


[Rust]: https://www.rust-lang.org/
[`just`]: https://crates.io/crates/just
[Kari code]: https://github.com/kari-lang/kari/tree/master/kr
[tests]: https://github.com/kari-lang/kari/tree/master/kr/tests
[builtins.rs]: https://github.com/kari-lang/kari/blob/master/src/builtins.rs
[Kari standard library]: https://github.com/kari-lang/kari/blob/master/kr/src/std.kr
