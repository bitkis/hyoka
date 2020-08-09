extern crate hyoka;

struct Env {}

impl Env {
    pub fn new() -> Self {
        Env {}
    }
}

#[derive(Debug, Clone)]
enum Expression {
    Float(f32),
    Symbol(String),
    List(Vec<Expression>),
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        if let Ok(v) = value.parse::<f32>() {
            Expression::Float(v)
        } else {
            Expression::Symbol(String::from(value))
        }
    }
}

impl Expression {
    pub fn parse(tokens: &mut Vec<String>) -> Result<Expression, String> {
        if tokens.len() == 0 {
            Err(format!("unexpected EOF while parsing the tokens."))
        } else {
            let token = tokens.remove(0);
            match token.as_str() {
                "(" => {
                    // TODO: Can we avoid ITM here?
                    let mut list = Vec::<Expression>::new();
                    while tokens[0] != ")" {
                        match Expression::parse(tokens) {
                            Ok(v) => list.push(v),
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    tokens.remove(0);
                    Ok(Expression::List(list))
                }
                ")" => Err(format!("unexpected ')' found while parsing tokens")),
                _ => Ok(Expression::from(token.as_str())),
            }
        }
    }

    pub fn evaluate(&mut self) -> Option<Expression> {
        match self.clone() {
            Expression::Float(_) => Some(self.clone()),
            Expression::Symbol(_) => Some(self.clone()),
            Expression::List(mut list) => {
                let car = list
                    .get(0)
                    .expect("Attempting to evaluate an empty Expression::List.");
                match car.clone() {
                    Expression::Symbol(procedure_name) => {
                        match procedure_name.as_str() {
                            // TODO: Make this a macro...
                            "+" => {
                                let result = list.iter_mut().skip(1).fold(0.0f32, |acc, x| match x
                                    .evaluate()
                                {
                                    Some(x) => match x {
                                        Expression::Float(x) => acc + x,
                                        _ => {
                                            panic!("All evaluation should yield Expression::Float")
                                        }
                                    },
                                    _ => panic!("Evaluation failed."),
                                });
                                Some(Expression::Float(result))
                            }
                            "-" => {
                                let result = list.iter_mut().skip(1).fold(0.0f32, |acc, x| match x
                                    .evaluate()
                                {
                                    Some(x) => match x {
                                        Expression::Float(x) => acc - x,
                                        _ => {
                                            panic!("All evaluation should yield Expression::Float")
                                        }
                                    },
                                    _ => panic!("Evaluation failed."),
                                });
                                Some(Expression::Float(result))
                            }
                            "*" => {
                                let result = list.iter_mut().skip(1).fold(1.0f32, |acc, x| match x
                                    .evaluate()
                                {
                                    Some(x) => match x {
                                        Expression::Float(x) => acc * x,
                                        _ => {
                                            panic!("All evaluation should yield Expression::Float")
                                        }
                                    },
                                    _ => panic!("Evaluation failed."),
                                });
                                Some(Expression::Float(result))
                            }
                            "/" => {
                                let result = list.iter_mut().skip(1).fold(1.0f32, |acc, x| match x
                                    .evaluate()
                                {
                                    Some(x) => match x {
                                        Expression::Float(x) => acc / x,
                                        _ => {
                                            panic!("All evaluation should yield Expression::Float")
                                        }
                                    },
                                    _ => panic!("Evaluation failed."),
                                });
                                Some(Expression::Float(result))
                            }
                            _ => {
                                println!("unknown procedure '{}' invoked.", procedure_name);
                                None
                            }
                        }
                    }
                    _ => {
                        println!("expected Expression::Symbol to represent procedure's name but found something else...");
                        None
                    }
                }
            }
        }
    }
}

pub fn main() {
    let mut repl = hyoka::Repl::new("lisp-calculator>", Env::new(), |_: &mut Env, y: String| {
        let mut y = y
            .replace('(', " ( ")
            .replace(')', " ) ")
            .split_ascii_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();
        match Expression::parse(&mut y) {
            Ok(mut result) => format!("result:{:?}", result.evaluate()),
            Err(e) => panic!(e),
        }
    });
    repl.run();
}
