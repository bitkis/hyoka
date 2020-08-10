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
                if env.global.contains_key(&name) {
                    match env.global.get(&name).unwrap() {
                        Expression::Float(x) => Some(Expression::Float(*x)),
                        Expression::Symbol(x) => {
                            let s = Expression::Symbol(x.clone());
                            // NOTE: This could potentially introduce infinite loop...
                            s.evaluate(env)
                        }
                        Expression::List(list) => {
                            let s = Expression::List(list.clone());
                            // NOTE: This could potentially introduce infinite loop...
                            s.evaluate(env)
                        }
                        Expression::Procedure(parameters, body) => {
                            let parameters = parameters
                                .iter()
                                .map(|expr| {
                                    if let Expression::Symbol(name) = expr {
                                        if let Some(env_expr) = env.global.get(name) {
                                            // NOTE: For now, all paramters must be floats because that is the only atomic types that we currently support.
                                            if let Expression::Float(value) = env_expr {
                                                Some((
                                                    Expression::Symbol(name.clone()),
                                                    Expression::Float(*value),
                                                ))
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<Option<(Expression, Expression)>>>();
                            if parameters.iter().any(Option::is_none) {
                                println!("some of the procedure's paramters are not floats.");
                                None
                            } else {
                                let parameters = parameters
                                    .iter()
                                    .map(|x| x.clone().unwrap())
                                    .collect::<Vec<(Expression, Expression)>>();
                                // Swap every symbol in the body with the unwrapped parameters above.
                                let body = body
                                    .iter()
                                    .map(|symbol| {
                                        if let Some((_, value)) =
                                            parameters.iter().find(|(key, _)| key == symbol)
                                        {
                                            value.clone()
                                        } else {
                                            symbol.clone()
                                        }
                                    })
                                    .collect::<Vec<Expression>>();
                                let body = Expression::List(body.clone());
                                body.evaluate(env)
                            }
                        }
                    }
                } else {
                    Some(Expression::Symbol(name.clone()))
                }
            }
            Expression::List(mut list) => {
                println!("list:{:?}", list);
                let car = list
                    .get(0)
                    .expect("Attempting to evaluate an empty Expression::List.");
                match car.clone() {
                    Expression::Symbol(procedure_name) => {
                        match procedure_name.as_str() {
                            // TODO: Make this a macro...
                            "+" => {
                                let result = list.iter_mut().skip(1).fold(0.0f32, |acc, x| match x
                                    .evaluate(env)
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
                                    .evaluate(env)
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
                                    .evaluate(env)
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
                                    .evaluate(env)
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
                            "lambda" => {
                                if list.len() != 3 {
                                    println!("`lambda` expects (<parameters> <body>)");
                                    None
                                } else {
                                    if let Expression::List(parameters) = list.get(1).unwrap() {
                                        if let Expression::List(body) = list.get(2).unwrap() {
                                            Some(Expression::Procedure(
                                                parameters.clone(),
                                                body.clone(),
                                            ))
                                        } else {
                                            println!(
                                                "a `lambda`'s body must be a list of expressions."
                                            );
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                }
                            }
                            "define" => {
                                if list.len() != 3 {
                                    println!("`define` takes (<name> <expression>)");
                                } else {
                                    let name = if let Expression::Symbol(name) =
                                        list.get(1).unwrap()
                                    {
                                        name.clone()
                                    } else {
                                        println!("a procedure's name must be a symbol. Aborting this definition.");
                                        return None;
                                    };
                                    let expression = list.get(2).unwrap().evaluate(env).unwrap();
                                    env.global.insert(name, expression);
                                }
                                // Defining a procedure simply mutates the global environment.
                                None
                            }
                            name => {
                                if let Some(Expression::Procedure(parameter, body)) =
                                    env.global.get(name)
                                {
                                    let args = list.iter().skip(1);
                                    if parameter.len() == args.len() {
                                        let mut env = env.clone();
                                        let mut local_env = args
                                            .zip(parameter.iter())
                                            .fold(Env::new(), |mut local_env, (value, name)| {
                                                let evaluated_value = value.evaluate(&mut env).unwrap();
                                                match name {
                                                    Expression::Symbol(name)=>{
                                                        println!("matching args name:{:?} evaluated_value:{:?}", name, evaluated_value);
                                                        local_env.global.insert(name.clone(), evaluated_value);
                                                    },
                                                    _=>{
                                                        println!("invalid expression for a symbol in parameter:{:?}",name)
                                                    }
                                                }
                                                local_env
                                            });
                                        println!("local_env:{:?}", local_env);
                                        let procedure_call = Expression::List(body.clone());
                                        procedure_call.evaluate(&mut local_env)
                                    } else {
                                        None
                                    }
                                } else {
                                    println!("unknown procedure '{}' invoked.", procedure_name);
                                    None
                                }
                            }
                        }
                    }
                    _ => {
                        println!("expected Expression::Symbol to represent procedure's name but found something else...");
                        None
                    }
                }
            }
            Expression::Procedure(parameters, body) => {
                println!(
                    r#"
                we have somehow stumbled across a procedure? I am interested in how this could happen.
                parameters:{:?}
                body:{:?}
                "#,
                    parameters, body
                );
                None
            }
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
        match Expression::parse(&mut y) {
            Ok(result) => format!("result:{:?} env:{:?}", result.evaluate(env), env),
            Err(e) => panic!(e),
        }
    });
    repl.run();
}
