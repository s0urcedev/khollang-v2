#[allow(dead_code)]
mod interpreter;

use std::io::{self, BufWriter};
use std::process::ExitCode;
use std::{env, fs, thread};

const USAGE: &str = "Usage: khol <file>";

/// The program runs on a thread with a big stack, so that deep recursion in the program
/// does not overflow the stack of the interpreter. A recursion of 1,000,000 calls has to
/// work ([design 5.7, 5.8](../.claude/docs/design.md)). A call level of the program takes
/// roughly 1.6 KiB of stack in a release build, and more when it is inside blocks, so the
/// first size gives a wide margin. The memory is only used when the recursion gets there. If
/// the system refuses a size, the next smaller one is tried.
const STACK_SIZES_MIB: [usize; 5] = [8192, 4096, 2048, 1024, 256];

type Outcome = Result<(), interpreter::error::Error>;

fn run_program(source: String) -> Option<std::thread::JoinHandle<Outcome>> {
    for mebibytes in STACK_SIZES_MIB {
        let source = source.clone();
        let spawned = thread::Builder::new()
            .stack_size(mebibytes * 1024 * 1024)
            .spawn(move || {
                let mut output = BufWriter::new(io::stdout().lock());
                interpreter::run(&source, &mut io::stdin().lock(), &mut output)
            });
        if let Ok(handle) = spawned {
            return Some(handle);
        }
    }
    None
}

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let Some(path) = arguments.next() else {
        eprintln!("khol: missing file operand\n{USAGE}");
        return ExitCode::from(2);
    };
    if let Some(extra) = arguments.next() {
        eprintln!("khol: extra operand '{extra}'\n{USAGE}");
        return ExitCode::from(2);
    }
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("khol: cannot read '{path}': {error}");
            return ExitCode::from(1);
        }
    };
    let Some(run) = run_program(source) else {
        eprintln!("khol: cannot start the interpreter: not enough memory for its stack");
        return ExitCode::from(1);
    };
    match run.join() {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(error)) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
        Err(_) => ExitCode::from(101),
    }
}
