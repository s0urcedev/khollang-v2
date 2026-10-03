# khollang-v2

This is a repository for version 2 of the Khollang educational programming language.

## Version 1

The original version 1 has an interpreter written in Python and its cloned repo can be found in `./v1`.

See `./v1/README.md` for the language description and `./v1/interpreter` for the interpreter implementation.

## V1 -> V2

This version rewrites the interpreter in **Rust**.

However, it is **NOT A DIRECT REWRITE**. The goal is to keep the same syntax and functionality of Khollang,
but in a newer interpreter that does not have to be an exact copy of the Python V1.

## Project structure

- `./.claude` – Claude files
- `./src` – main code
- `./src/interpreter` – interpreter implementation
- `./v1` – V1 code
- `./v1/interpreter` – V1 interpreter implementation
- `./docs` – documentation