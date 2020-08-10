extern crate hyoka;

use std::rc::Rc;

#[derive(Debug, Clone)]
struct Env {
    global: std::collections::HashMap<String, Expression>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            global: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Expression {
    Float(f32),
    Symbol(Rc<String>),
    List(Rc<Vec<Expression>>),
    // (Parameters, Body)
    Procedure(Rc<Vec<Expression>>, Rc<Vec<Expression>>),
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        if let Ok(v) = value.parse::<f32>() {
            Expression::Float(v)
        } else {
            Expression::Symbol(Rc::new(String::from(value)))
        }
    }
}

impl Expression {
    pub fn parse(tokens: &mut Vec<String>) -> Result<Expression, String> {
        if tokens.is_empty() {
            Err("unexpected EOF while parsing the tokens.".to_string())
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
                    Ok(Expression::List(Rc::new(list)))
                }
                ")" => Err("unexpected ')' found while parsing tokens".to_string()),
                _ => Ok(Expression::from(token.as_str())),
            }
        }
    }

    pub fn evaluate(&self, env: &mut Env) -> Option<Expression> {
        match self {
            Expression::Float(_) => Some(self.clone()),
            Expression::Symbol(name) => match env.global.get(name.as_ref()) {
                Some(Expression::Float(x)) => Some(Expression::Float(*x)),
                // TODO: This seems dubious, it ought to be possible to recursively call evaluate without having to copy?
                Some(Expression::Symbol(x)) => Expression::Symbol(x.clone()).evaluate(env),
                Some(Expression::List(list)) => Expression::List(list.clone()).evaluate(env),
                Some(Expression::Procedure(parameter_symbols, body)) => Some(
                    Expression::Procedure(parameter_symbols.clone(), body.clone()),
                ),
                None => Some(Expression::Symbol(name.clone())),
            },
            Expression::List(list) => {
                match &list.as_slice() {
                    [Expression::Symbol(procedure_name), ref args @ ..] => {
                        match procedure_name.as_str() {
                            // TODO: Make this a macro...
                            "+" => {
                                let result = args.iter().fold(0.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc + x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "-" => {
                                let result = args.iter().fold(0.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc - x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "*" => {
                                let result = args.iter().fold(1.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc * x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "/" => {
                                let result = args.iter().fold(1.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc / x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "lambda" => {
                                if let [_, Expression::List(params), Expression::List(body)] =
                                    list.as_slice()
                                {
                                    Some(Expression::Procedure(params.clone(), body.clone()))
                                } else {
                                    None
                                }
                            }
                            "define" => {
                                if let [_, Expression::Symbol(name), ref expression] =
                                    list.as_slice()
                                {
                                    if let Some(evaluation) = expression.evaluate(env) {
                                        env.global.insert(name.as_ref().clone(), evaluation);
                                    }
                                }
                                None
                            }
                            name => {
                                if let Some(Expression::Procedure(parameter, body)) =
                                    env.global.get(name)
                                {
                                    if parameter.len() == args.len() {
                                        let mut env = env.clone();
                                        let mut local_env = args.iter().zip(parameter.iter()).fold(
                                            Env::new(),
                                            |mut local_env, (arg_body, arg_symbol)| {
                                                let value = arg_body.evaluate(&mut env).unwrap();
                                                if let Expression::Symbol(arg_symbol) = arg_symbol {
                                                    local_env
                                                        .global
                                                        .insert(arg_symbol.as_ref().clone(), value);
                                                }
                                                local_env
                                            },
                                        );
                                        Expression::List(body.clone()).evaluate(&mut local_env)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                        }
                    }
                    [..] => None,
                }
            }
            // NOTE: What _should_ happen here?
            Expression::Procedure(_, _) => None,
        }
    }
}

pub fn main() {
    let mut repl = hyoka::Repl::new("lisp>", Env::new(), |env: &mut Env, y: String| {
        let mut y = y
            .replace('(', " ( ")
            .replace(')', " ) ")
            .split_ascii_whitespace()
            .map(String::from)
            .collect::<Vec<String>>();
        if let Ok(Some(Expression::Float(result))) = Expression::parse(&mut y)
            .map(|x| x.evaluate(env))
            .map_err(|x| {
                println!("error:{}", x);
            })
        {
            Some(format!("{}", result))
        } else {
            None
        }
    });
    repl.run();
}
