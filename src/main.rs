use std::{
    env,
    ffi::OsString,
    fs::File,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use minipy::Interpreter;

fn get_script_path() -> Option<PathBuf> {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    }
}

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::build();
    match get_script_path() {
        Some(script_path) => {
            let interpreter = Arc::new(interpreter);
            interpreter.clone().init_builtin_types();

            let file = File::open(script_path)?;
            let script = io::read_to_string(io::BufReader::new(file))?;
            let lines: Vec<&str> = script.lines().collect();

            let mut result = Ok(());
            for line in lines {
                if let Err(e) = interpreter.clone().eval_line(line) {
                    result = Err(io::Error::other(format!("Unhandled error occurred: {}", e)));
                    break;
                }
            }

            if result.is_ok() {
                result = interpreter.end_eval().map_err(io::Error::other);
            }

            result
        }
        None => {
            interpreter.open_repl_output();
            let interpreter = Arc::new(interpreter);
            interpreter.clone().init_builtin_types();

            loop {
                print!("mini-py> ");
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if let Err(e) = interpreter.clone().eval_line(&input) {
                    eprintln!("Unhandled error: {}", e);
                    break Err(io::Error::other("Unhandled error occurred"));
                }
            }
        }
    }
}

// Known issues:
// mini-py> a = 1
// mini-py> if a:
// mini-py>   b=2
// mini-py>   if b == 2:
// mini-py>     a+b
// mini-py>
// mini-py> a=3
// Unhandled error: Error evaluating line: Unexpected indent.
// Error: Custom { kind: Other, error: "Unhandled error occurred" }
