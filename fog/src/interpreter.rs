use std::collections::HashMap;
use std::rc::Rc;

use crate::ast_nodes::Statement::*;
use crate::ast_nodes::*;

// --- variables & types ---

#[derive(Clone)]
pub struct Variable {
    pub value: Option<Value>,
    pub r#type: Rc<Type>,
}

#[derive(Clone)]
pub enum Value {
    Type(Rc<Type>),
    Int32(i32),
    Float32(f32),
    Lambda(Rc<Identifier>, Rc<Expr>)
}

#[derive(Clone)]
pub enum Type {
    Kind,
    Primitive(String),
    Function(Rc<Type>, Rc<Type>),
}

// --- environment ---

pub struct Environment {
    pub variables: HashMap<String, Variable>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    fn annotate_var_type(&mut self, name: &str, r#type: Rc<Type>) -> Result<(), InterpreterError> {
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
                value: None,
                r#type: r#type,
            },
        );

        Ok(())
    }

    fn declare_var(&mut self, name: &str, value: Value) -> Result<(), InterpreterError> {
        let existing: &mut Variable = match self.variables.get_mut(name) {
            Some(var) => var,
            None => {
                return Err(InterpreterError {
                    message: format!("variable `{}` not declared in the current scope", name),
                });
            }
        };

        existing.value = Some(value);

        Ok(())
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

        let r#type: Type = Type::Primitive("Type".to_string());
        let rc_type: Rc<Type> = Rc::new(r#type);

        interpreter.top_env.variables.insert(
            "Type".to_string(),
            Variable {
                value: Some(Value::Type(rc_type.clone())),
                r#type: rc_kind.clone(),
            },
        );

        interpreter.top_env.variables.insert(
            "Int32".to_string(),
            Variable {
                value: Some(Value::Type(Rc::new(Type::Primitive("Int32".to_string())))),
                r#type: rc_type.clone(),
            },
        );

        interpreter.top_env.variables.insert(
            "Float32".to_string(),
            Variable {
                value: Some(Value::Type(Rc::new(Type::Primitive("Float32".to_string())))),
                r#type: rc_type.clone(),
            },
        );

        interpreter
    }

    pub fn run(program: Box<Program>) {
        let mut interpreter = Interpreter::new(program);
        let statements = std::mem::take(&mut interpreter.program.statements);

        for stmt in statements {
            match stmt {
                TypeAnnotation(ident, expr) => interpreter.annotate_type(&ident, &expr),
                Declaration(ident, expr) => interpreter.declare(&ident, &expr),
            };
        }
    }

    fn annotate_type(&mut self, ident: &Identifier, expr: &Expr) {
        self.top_env.annotate_var_type(name, r#type)
    }

    fn declare(&mut self, ident: &Identifier, expr: &Expr) {}

    fn eval_expr(expr: &Expr) -> Value {
        Value::Type(...);
    }
}
