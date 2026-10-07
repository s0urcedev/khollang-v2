pub mod ast;
pub mod builtins;
pub mod context;
pub mod error;
pub mod executor;
pub mod input;
pub mod lexer;
pub mod limits;
pub mod ops;
pub mod parser;
pub mod scope;
pub mod token;
pub mod value;

use std::io::{BufRead, Write};

use context::Context;
use error::Error;
use limits::{Limits, Tracker};
use scope::{ScopeKind, new_scope};

/// The deepest depth that a call of the program's functions and procedures may be at. The
/// first call, made at the top level, is at depth 0, so 1,000,001 calls can be nested. A
/// deeper call is a runtime error ([design 5.7](../../.claude/docs/design.md)).
pub const MAX_CALL_DEPTH: usize = 1_000_000;

/// Runs a program: lexes, parses and executes it. `input` and `output` are standard input
/// and standard output of the program.
pub fn run(source: &str, input: &mut dyn BufRead, output: &mut dyn Write) -> Result<(), Error> {
    run_with_limits(source, input, output, Limits::none())
}

/// [`run`] with limits on what the program may use ([design 6](../../.claude/docs/design.md)).
pub fn run_with_limits(
    source: &str,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    limits: Limits,
) -> Result<(), Error> {
    run_configured(source, input, output, limits, MAX_CALL_DEPTH)
}

/// [`run`] with another limit on the depth of calls.
pub fn run_with_limit(
    source: &str,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    max_depth: usize,
) -> Result<(), Error> {
    run_configured(source, input, output, Limits::none(), max_depth)
}

fn run_configured(
    source: &str,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    limits: Limits,
    max_depth: usize,
) -> Result<(), Error> {
    let (lines, statements) = lexer::lex_counted(source)?;
    let program = parser::parse(lines)?;
    let mut statements = statements;
    statements.extend(limits::declarations(&program));
    // in the order of the source, for the first statement that goes over a limit
    statements.sort_by_key(|statement| (statement.line, statement.column));
    limits::check_static(&statements, &limits)?;
    let declared = limits::totals(&statements);
    let mut context = Context {
        scope: new_scope(ScopeKind::Global, None),
        return_type: None,
        depth: 0,
        max_depth,
        tracker: Tracker::new(limits, &declared),
        input,
        output,
    };
    let result = executor::run_statements(&program, &mut context);
    // what the program wrote stays, also after an error
    let flushed = context.output.flush();
    result?;
    flushed.map_err(|e| Error::new(1, None, format!("cannot write the output: {e}")))
}

#[cfg(test)]
mod tests;
