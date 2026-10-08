# Khollang

Khollang is an educational programming language with a simple pseudocode-like syntax and functionality to limit which parts of the language a program can use.

This is version 2, an interpreter written in Rust. Version 1, written in Python, is at [github.com/s0urcedev/khollang](https://github.com/s0urcedev/khollang).

## Installation

With [Rust](https://www.rust-lang.org/tools/install) installed, run this in the repository:

```
cargo install --path .
```

It builds the `khol` executable and puts it in `~/.cargo/bin`. Run the same command again to update it.

## Usage

```
khol program.kh [limits-file]
```

See `khol --help` for details.

```
// hello.kh
input NAME
output "Hello, " + NAME + "!"
```

## Documentation

- [`docs/guide.md`](docs/guide.md): the language guide, including limits
- [`docs/examples`](docs/examples): ten example programs that show the language feature by feature

## VS Code

Syntax highlighting for `.kh` files is in [`editors/vscode`](editors/vscode). See its [README](editors/vscode/README.md) for how to install it.
