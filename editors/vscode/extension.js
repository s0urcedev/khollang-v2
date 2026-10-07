// Runs the open Khollang file with `khol` in a terminal.

const vscode = require("vscode");

const TERMINAL_NAME = "Khollang";

/** Quotes a path for the shell of the terminal. */
function quote(path) {
    if (process.platform === "win32") {
        return `"${path}"`;
    }
    return `'${path.replace(/'/g, `'\\''`)}'`;
}

/** The terminal of an earlier run, or a new one. */
function terminal() {
    const existing = vscode.window.terminals.find(
        (t) => t.name === TERMINAL_NAME && t.exitStatus === undefined
    );
    return existing ?? vscode.window.createTerminal(TERMINAL_NAME);
}

async function run() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== "khollang") {
        vscode.window.showErrorMessage("Open a Khollang file to run it.");
        return;
    }
    const document = editor.document;
    if (document.isUntitled) {
        vscode.window.showErrorMessage("Save the file before running it.");
        return;
    }
    if (document.isDirty && !(await document.save())) {
        return;
    }
    const shell = terminal();
    shell.show();
    shell.sendText(`khol ${quote(document.uri.fsPath)}`);
}

function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand("khollang.run", run));
}

function deactivate() {}

module.exports = { activate, deactivate };
