use std::collections::HashMap;
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
    Function {
        param: String,
        body: Rc<Expr>,
        captured_env: Box<Environment>,
    },
    NativeFunction(Rc<dyn Fn(Value) -> Result<Value, InterpreterError>>),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Type(r#type) => (*r#type).to_string(),
            Value::Int32(value) => value.to_string(),
            Value::Float32(value) => value.to_string(),
            Value::Function { param, body, .. } => {
                format!("{} => {}", param, (*body).to_string())
            }
            Value::NativeFunction(_) => "[native function]".to_string(),
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
        (Value::Function { .. }, Type::Function(_, _)) => true,
        _ => false,
    }
}

// --- environment ---

#[derive(Clone)]
pub struct Environment {
    pub variables: HashMap<String, Variable>,
    pub parent: Option<Box<Environment>>,
}

impl Environment {
    fn annotate_type(&mut self, name: &str, r#type: Rc<Type>) -> Result<(), InterpreterError> {
        if self.variables.contains_key(name) {
            return Err(InterpreterError::from_string(format!(
                "variable `{}` already annotated its type in the scope",
                name
            )));
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
        let var: &mut Variable = self.variables.get_mut(name).ok_or_else(|| {
            InterpreterError::from_string(format!(
                "variable `{}` not found in the current scope",
                name
            ))
        })?;

        if var.value.is_some() {
            return Err(InterpreterError::from_string(format!(
                "variable `{}` already declared in the current scope",
                name
            )));
        }

        if !is_value_of_type(&value, &var.r#type) {
            return Err(InterpreterError::from_string(format!(
                "type mismatch when assigning to variable `{}`",
                name
            )));
        }

        var.value = Some(value);
        Ok(())
    }

    fn get_var(&self, name: &str) -> Result<Value, InterpreterError> {
        if let Some(var) = self.variables.get(name) {
            return var.value.clone().ok_or_else(|| {
                InterpreterError::from_string(format!("variable `{}` not declared", name))
            });
        }

        if let Some(parent) = &self.parent {
            return parent.get_var(name);
        }

        Err(InterpreterError::from_string(format!(
            "variable `{}` not found in the current scope",
            name
        )))
    }
}

// --- eval & helpers ---

fn eval_expr(expr: &Expr, env: &Environment) -> Result<Value, InterpreterError> {
    match expr {
        // literals
        Expr::Int32Literal(value) => Ok(Value::Int32(*value)),
        Expr::Float32Literal(value) => Ok(Value::Float32(*value)),

        // variable
        Expr::Identifier(name) => match env.get_var(&name) {
            Ok(value) => Ok(value),
            Err(error) => Err(error),
        },

        // AST lambda -> interpreter function
        Expr::Lambda { param, body } => Ok(Value::Function {
            param: param.clone(),
            body: Rc::clone(body),
            captured_env: Box::new(env.clone()),
        }),

        // function application
        Expr::FuncAppl {
            function: function_name,
            args: arguments,
        } => {
            let mut result: Value = eval_expr(&Expr::Identifier(function_name.clone()), env)?;
            for arg in arguments {
                result = apply_function(result, eval_expr(arg, env)?)?;
            }
            Ok(result)
        }
    }
}

fn apply_function(function: Value, argument: Value) -> Result<Value, InterpreterError> {
    match function {
        Value::Function {
            param: parameter_name,
            body,
            captured_env,
        } => {
            let mut child_env: Environment = Environment {
                variables: HashMap::new(),
                parent: Some(captured_env),
            };

            child_env.variables.insert(
                parameter_name.clone(),
                Variable {
                    name: parameter_name.clone(),
                    value: Some(argument),
                    r#type: Rc::new(Type::Kind), // placeholder — real type checking comes later
                },
            );

            eval_expr(&body, &child_env)
        }

        Value::NativeFunction(f) => f(argument),

        _ => Err(InterpreterError::from_str(
            "cannot apply a non-function value",
        )),
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
        let rc_type: Rc<Type> = Rc::new(Type::Type);

        let rc_i32: Rc<Type> = Rc::new(Type::Int32);
        let rc_i32_i32: Rc<Type> = Rc::new(Type::Function(rc_i32.clone(), rc_i32.clone()));
        let rc_i32_i32_i32: Rc<Type> = Rc::new(Type::Function(rc_i32.clone(), rc_i32_i32.clone()));

        let var_type: Variable = Variable {
            name: "Type".to_string(),
            value: Some(Value::Type(rc_type.clone())),
            r#type: rc_kind.clone(),
        };

        let var_int32: Variable = Variable {
            name: "Int32".to_string(),
            value: Some(Value::Type(Rc::new(Type::Int32))),
            r#type: rc_type.clone(),
        };

        let var_float32: Variable = Variable {
            name: "Float32".to_string(),
            value: Some(Value::Type(Rc::new(Type::Float32))),
            r#type: rc_type.clone(),
        };

        let var_plus_int_int: Variable = Variable {
            name: "+".to_string(),
            value: Some(Value::NativeFunction(Rc::new(|a| match a {
                Value::Int32(lhs) => Ok(Value::NativeFunction(Rc::new(move |b| match b {
                    Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                    _ => Err(InterpreterError::from_str("right operand is not an Int32")),
                }))),
                _ => Err(InterpreterError::from_str("left operand is not an Int32")),
            }))),
            r#type: rc_i32_i32_i32.clone(),
        };

        vec![var_type, var_int32, var_float32, var_plus_int_int]
            .iter()
            .for_each(|var| {
                interpreter
                    .top_env
                    .variables
                    .insert(var.name.clone(), var.clone());
            });

        interpreter
    }

    pub fn run(program: Box<Program>) {
        let mut interpreter: Interpreter = Interpreter::new(program);
        let mut errors: Vec<InterpreterError> = Vec::new();

        let top_env: &mut Environment = &mut interpreter.top_env;

        for stmt in interpreter.program.statements {
            match stmt {
                TypeAnnotation(name, expr) => match annotate_type(&name, &expr, top_env) {
                    Ok(_) => (),
                    Err(error) => errors.push(error),
                },
                Declaration(name, expr) => match declare(&name, &expr, top_env) {
                    Ok(_) => (),
                    Err(error) => errors.push(error),
                },
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
    let value: Value = match eval_expr(expr, env) {
        Ok(value) => value,
        Err(error) => return Err(error),
    };

    (*env).declare(name, value)
}
