use std::io::prelude::*;

pub struct Repl<S, F>
    where
        F: Fn(&mut S, String) -> String,
{
    state: S,
    print: &'static str,
    evaluate: F,
}

impl<S, F> Repl<S, F>
    where
        F: Fn(&mut S, String) -> String,
{
    pub fn new(print: &'static str, state: S, evaluate: F) -> Self {
        Self {
            state,
            evaluate,
            print,
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("{}", self.print);
            std::io::stdout().flush().unwrap();

            let input = {
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .ok()
                    .expect("Failed to read line");
                input
            };

            // 1. Match the input against any internal commands that we have
            // 2. Else, evaluate.
            match input.as_str() {
                // TODO: In the future, we might have an additional checks here if the user also defined/override the internal commands.
                "exit" | "quit" => break,
                "clear" => unimplemented!(
                    "'clear' is currently not implemented. It should clear the screen."
                ),
                _ => {
                    println!("{}", (self.evaluate)(&mut self.state, input));
                }
            }
        }
    }
}
