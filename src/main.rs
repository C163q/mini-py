use std::{
    io::{self, Write},
    sync::Arc,
};

use minipy::Interpreter;

fn main() -> io::Result<()> {
    let mut interpreter = Interpreter::build();
    interpreter.open_repl_output();
    let interpreter = Arc::new(interpreter);
    interpreter.clone().init_builtin_types();
    loop {
        print!("mini-py> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        interpreter
            .clone()
            .eval_line(&input)
            .expect("Unrecoverable error during evaluation");
    }
}
