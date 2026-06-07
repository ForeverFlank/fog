use std::collections::HashMap;
use std::fmt::{Display, format};
use std::rc::Rc;

use crate::ast_nodes::Statement::*;
use crate::ast_nodes::*;

// --- variables & types ---

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub value: Option<Value>,
    pub r#type: Rc<Type>,
}

#[derive(Clone)]
pub enum Value {
    Type(Rc<Type>),
    Int32(i32),
    Float32(f32),
    Function(Rc<Identifier>, Rc<Expr>),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Type(r#type) => (*r#type).to_string(),
            Value::Int32(value) => value.to_string(),
            Value::Float32(value) => value.to_string(),
            Value::Function(param, expr) => format!("{} => {}", param.0, (*expr).to_string()),
        }
    }
}

#[derive(Clone)]
pub enum Type {
    Kind,
    Type,
    Int32,
    Float32,
    Function(Rc<Type>, Rc<Type>),
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Kind => "Kind".to_string(),
            Type::Type => "Type".to_string(),
            Type::Int32 => "Int32".to_string(),
            Type::Float32 => "Float32".to_string(),
            Type::Function(domain, codomain) => {
                format!("{} -> {}", (*domain).to_string(), (*codomain).to_string())
            }
        }
    }
}

fn is_value_of_type(value: &Value, r#type: &Type) -> bool {
    match (value, r#type) {
        (Value::Type(_), Type::Type) => true,
        (Value::Int32(_), Type::Int32) => true,
        (Value::Float32(_), Type::Float32) => true,
        (Value::Function(_, _), Type::Function(_, _)) => true,
        _ => false,
    }
}

// --- environment ---

pub struct Environment {
    pub variables: HashMap<String, Variable>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    fn annotate_type(&mut self, name: &str, r#type: Rc<Type>) -> Result<(), InterpreterError> {
        if self.variables.contains_key(name) {
            return Err(InterpreterError {
                message: format!(
                    "variable `{}` already annotated its type in the scope",
                    name
                ),
            });
        }

        self.variables.insert(
            name.to_string(),
            Variable {
                name: name.to_string(),
                value: None,
                r#type: r#type,
            },
        );

        Ok(())
    }

    fn declare(&mut self, name: &str, value: Value) -> Result<(), InterpreterError> {
        let existing: Option<&mut Variable> = self.variables.get_mut(name);

        match existing {
            Some(var) => match var.value {
                Some(_) => Err(InterpreterError {
                    message: format!("variable `{}` already declared in the current scope", name),
                }),
                None => {
                    var.value = Some(value);
                    Ok(())
                }
            },
            None => Err(InterpreterError {
                message: format!("variable `{}` already declared in the current scope", name),
            }),
        }
    }

    fn get_var(&self, name: &str) -> Result<Value, InterpreterError> {
        match self.variables.get(name) {
            Some(var) => match &var.value {
                Some(value) => {
                    if is_value_of_type(value, &var.r#type) {
                        Ok(value.clone())
                    } else {
                        Err(InterpreterError::from_string(format!(
                            "type mismatch when assigning to variable `{}`",
                            name
                        )))
                    }
                }
                None => Err(InterpreterError {
                    message: format!("variable `{}` already declared in the current scope", name),
                }),
            },
            None => Err(InterpreterError::from_string(format!(
                "variable `{}` already declared in the current scope",
                name
            ))),
        }
    }
}

// --- eval & helpers ---

fn eval_expr(expr: &Expr, env: &Environment) -> Result<Value, InterpreterError> {
    match expr {
        Expr::Identifier(ident) => match env.get_var(&ident.0) {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        },
        Expr::Int32Literal(value) => Ok(Value::Int32(*value)),
        Expr::Float32Literal(value) => Ok(Value::Float32(*value)),
        // Expr::FuncAppl(FuncAppl { function, arguments }) => Ok()
        _ => Err(InterpreterError::from_str("Unsupported expression")),
    }
}

// --- interpreter ---

pub struct Interpreter {
    pub program: Box<Program>,
    pub top_env: Environment,
}

pub struct InterpreterError {
    pub message: String,
}

impl InterpreterError {
    fn from_str(message: &str) -> InterpreterError {
        InterpreterError {
            message: message.to_string(),
        }
    }

    fn from_string(message: String) -> InterpreterError {
        InterpreterError { message: message }
    }
}

pub fn run(program: Box<Program>) {
    Interpreter::run(program);
}

impl Interpreter {
    fn new(program: Box<Program>) -> Interpreter {
        let mut interpreter: Interpreter = Interpreter {
            program,
            top_env: Environment {
                variables: HashMap::new(),
                parent: None,
            },
        };

        let rc_kind: Rc<Type> = Rc::new(Type::Kind);

        let r#type: Type = Type::Type;
        let rc_type: Rc<Type> = Rc::new(r#type);

        interpreter.top_env.variables.insert(
            "Type".to_string(),
            Variable {
                name: "Type".to_string(),
                value: Some(Value::Type(rc_type.clone())),
                r#type: rc_kind.clone(),
            },
        );

        interpreter.top_env.variables.insert(
            "Int32".to_string(),
            Variable {
                name: "Int32".to_string(),
                value: Some(Value::Type(Rc::new(Type::Int32))),
                r#type: rc_type.clone(),
            },
        );

        interpreter.top_env.variables.insert(
            "Float32".to_string(),
            Variable {
                name: "Float32".to_string(),
                value: Some(Value::Type(Rc::new(Type::Float32))),
                r#type: rc_type.clone(),
            },
        );

        interpreter
    }

    pub fn run(program: Box<Program>) {
        let mut interpreter: Interpreter = Interpreter::new(program);
        let statements: Vec<Statement> = std::mem::take(&mut interpreter.program.statements);
        let mut errors: Vec<InterpreterError> = Vec::new();

        for stmt in statements {
            match stmt {
                TypeAnnotation(ident, expr) => {
                    match annotate_type(&ident.0, &expr, &mut interpreter.top_env) {
                        Ok(_) => (),
                        Err(error) => errors.push(error),
                    }
                }
                Declaration(ident, expr) => {
                    match declare(&ident.0, &expr, &mut interpreter.top_env) {
                        Ok(_) => (),
                        Err(error) => errors.push(error),
                    }
                }
            };
        }

        let mut all_vars: Vec<Variable> = interpreter.top_env.variables.values().cloned().collect();
        all_vars.sort_by(|a, b| a.name.cmp(&b.name));

        println!();
        for var in all_vars {
            println!(
                "{} : {} = {}",
                var.name,
                var.r#type.to_string(),
                match var.value {
                    Some(value) => value.to_string(),
                    None => "?".to_string(),
                }
            );
        }
        println!();

        for error in errors {
            println!("error: {}", error.message)
        }
    }
}

fn annotate_type(name: &str, expr: &Expr, env: &mut Environment) -> Result<(), InterpreterError> {
    let r#type: Rc<Type> = match eval_expr(&expr, env) {
        Ok(value) => match value {
            Value::Type(r#type) => r#type,
            _ => {
                return Err(InterpreterError::from_str("expression is not a type"));
            }
        },
        Err(error) => return Err(error),
    };

    (*env).annotate_type(name, r#type)
}

fn declare(name: &str, expr: &Expr, env: &mut Environment) -> Result<(), InterpreterError> {
    let value = match eval_expr(expr, env) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    (*env).declare(name, value)
}
