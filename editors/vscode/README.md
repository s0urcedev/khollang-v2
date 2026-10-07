# Khollang for VS Code

Syntax highlighting for [Khollang](../../README.md) programs (`.kh` files): keywords, strings and escapes, numbers, operators, comments, function and method calls, and:

- variables, with constants, parameters and structure attributes told apart where they are declared;
- data types: built-in types, and structure names wherever they are used as a type (`Point P`, `Array<Point>`, `function Point F(Point A)`), with their type arguments.

It also sets up `//` comments, bracket pairs and indentation after block headers.

## Running a file

The ▶ button in the editor title bar, or **Khollang: Run File** in the Command Palette, saves the open `.kh` file and runs `khol <file>` in a terminal named "Khollang". `khol` has to be on the `PATH` ([installation](../../README.md#installation)). The program's input is typed into that terminal.

## Installation

Package this folder into the VS Code and install the `.vsix` file:

```
npx @vscode/vsce package
code --install-extension "khollang-$(node -p "require('./package.json').version").vsix"
```
