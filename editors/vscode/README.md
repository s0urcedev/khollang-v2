# Khollang for VS Code

Syntax highlighting for [Khollang](../../README.md) programs (`.kh` files): keywords, strings and escapes, numbers, operators, comments, function and method calls, and:

- variables, with constants, parameters and structure attributes told apart where they are declared;
- data types: built-in types, and structure names wherever they are used as a type (`Point P`, `Array<Point>`, `function Point F(Point A)`), with their type arguments.

It also sets up `//` comments, bracket pairs and indentation after block headers.

## Installation

Link this folder into the VS Code extensions folder and restart VS Code:

```
ln -sfn "$(pwd)" ~/.vscode/extensions/khollang
```

Or package it and install the `.vsix` file:

```
npx @vscode/vsce package
code --install-extension khollang-1.0.0.vsix
```

To try changes, open this folder in VS Code and press **F5**: a second window opens with the extension loaded. **Developer: Inspect Editor Tokens and Scopes** shows how any piece of text is highlighted.
