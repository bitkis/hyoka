extern crate hyoka;

use std::convert::TryFrom;

struct Env {
    global: std::collections::HashMap<String, Expression>
}

impl Default for Env {
    fn default() -> Self {
        Env {
            global: std::collections::HashMap::new()
        }
    }
}

impl Env {}

#[derive(Debug, Clone)]
enum Expression {
    Float(f32),
    Integer(u32),
    Symbol(String),
    List(Vec<Expression>),
}

impl TryFrom<&str> for Expression {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(v) = value.parse::<u32>() {
            Ok(Expression::Integer(v))
        } else if let Ok(v) = value.parse::<f32>() {
            Ok(Expression::Float(v))
        } else {
            Ok(Expression::Symbol(String::from(value)))
        }
    }
}

impl Expression {
    pub fn parse(tokens: &mut Vec<String>) -> Result<Expression, String> {
        println!("parsing:{:?}", tokens);
        if tokens.len() == 0 {
            Err(format!("unexpected EOF while parsing the tokens."))
        } else {
            let token = tokens.remove(0);
            match token.as_str() {
                "(" => {
                    let mut L = Vec::<Expression>::new();
                    while tokens[0] != ")" {
                        match Expression::parse(tokens) {
                            Ok(v) => L.push(v),
                            Err(e) => { return Err(e); }
                        }
                    }
                    tokens.remove(0);
                    Ok(Expression::List(L))
                }
                ")" => Err(format!("unexpected ')' found while parsing tokens")),
                _ => Expression::try_from(token.as_str())
            }
        }
    }
}


pub fn main() {
    let mut repl = hyoka::Repl::new("lisp-summation>", Env::default(), |x: &mut Env, y: String| {
        let mut y = y.replace('(', " ( ").replace(')', " ) ").split_ascii_whitespace().
            map(String::from).collect::<Vec<String>>();
        match Expression::parse(&mut y) {
            Ok(result) => format!("result:{:?}", result),
            Err(e) => panic!(e)
        }
    });
    repl.run();
}