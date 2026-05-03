use std::io::{self, Write};

fn main() -> io::Result<()> {
    loop {
        print!("mini-py> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        println!("{}", input.trim_end());
    }
}
