extern crate hyoka;

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
    Symbol(String),
    List(Vec<Expression>),
    // (Parameters, Body)
    Procedure(Vec<Expression>, Vec<Expression>),
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

    pub fn evaluate(&self, env: &mut Env) -> Option<Expression> {
        match self.clone() {
            Expression::Float(_) => Some(self.clone()),
            Expression::Symbol(name) => {
                match env.global.get(&name) {
                    Some(Expression::Float(x)) => Some(Expression::Float(*x)),
                    Some(Expression::Symbol(x)) => {
                        let s = Expression::Symbol(x.clone());
                        // NOTE: This could potentially introduce infinite loop...
                        s.evaluate(env)
                    }
                    Some(Expression::List(list)) => {
                        let s = Expression::List(list.clone());
                        // NOTE: This could potentially introduce infinite loop...
                        s.evaluate(env)
                    }
                    // NOTE: WTF Was I trying to do here????
                    Some(Expression::Procedure(parameter_symbols, body)) => Some(
                        Expression::Procedure(parameter_symbols.clone(), body.clone()),
                    ),
                    None => Some(Expression::Symbol(name.clone())),
                }
            }
            Expression::List(mut list) => {
                match list.as_mut_slice() {
                    [Expression::Symbol(procedure_name), args @ ..] => {
                        match procedure_name.as_str() {
                            // TODO: Make this a macro...
                            "+" => {
                                let result = args.iter_mut().fold(0.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc + x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "-" => {
                                let result = args.iter_mut().fold(0.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc - x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "*" => {
                                let result = args.iter_mut().fold(1.0f32, |acc, x| {
                                    if let Some(Expression::Float(x)) = x.evaluate(env) {
                                        acc * x
                                    } else {
                                        acc
                                    }
                                });
                                Some(Expression::Float(result))
                            }
                            "/" => {
                                let result = args.iter_mut().fold(1.0f32, |acc, x| {
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
                                if let [_, Expression::Symbol(name), expression] = list.as_slice() {
                                    if let Some(evaluation) = expression.evaluate(env) {
                                        env.global.insert(name.clone(), evaluation);
                                    }
                                }
                                None
                            }
                            name => {
                                println!(
                                    "calling procedure\nname:{:#?}\nargs:{:#?}\nbody:{:#?}",
                                    name,
                                    args,
                                    env.global.get(name)
                                );
                                if let Some(Expression::Procedure(parameter, body)) =
                                    env.global.get(name)
                                {
                                    if parameter.len() == args.len() {
                                        let mut env = env.clone();
                                        let mut local_env = args.iter().zip(parameter.iter()).fold(
                                            Env::new(),
                                            |mut local_env, (arg_body, arg_symbol)| {
                                                let value = arg_body.evaluate(&mut env).unwrap();
                                                println!(
                                                    "evaluated\narg_body:{:#?}\nvalue:{:#?}",
                                                    arg_body, value
                                                );
                                                if let Expression::Symbol(arg_symbol) = arg_symbol {
                                                    local_env
                                                        .global
                                                        .insert(arg_symbol.clone(), value);
                                                }
                                                local_env
                                            },
                                        );
                                        let procedure_call = Expression::List(body.clone());
                                        procedure_call.evaluate(&mut local_env)
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
        let result = if let Ok(Some(Expression::Float(result))) =
            Expression::parse(&mut y).map(|x| x.evaluate(env))
        {
            Some(format!("{}", result))
        } else {
            None
        };
        println!("env:{:#?}", env);
        result
    });
    repl.run();
}
