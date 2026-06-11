use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::nodes::Expr;
use crate::ast::nodes::Program;
use crate::ast::nodes::Statement::*;
use crate::interpreter::environment::Environment;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;

pub struct Interpreter {
    pub program: Box<Program>,
    pub top_env: Environment,
    pub type_interner: TypeInterner,
}

pub struct TypeInterner {
    pub type_map: HashMap<Type, TypeId>,
    pub total_type_id: TypeId,
}

pub type TypeId = i32;

impl TypeInterner {
    pub fn new() -> TypeInterner {
        TypeInterner {
            type_map: HashMap::new(),
            total_type_id: 0,
        }
    }

    pub fn get_or_add_type(&mut self, r#type: Type) -> TypeId {
        if let Some(id) = self.type_map.get(&r#type) {
            *id
        } else {
            let id: i32 = self.total_type_id;
            self.type_map.insert(r#type, id);
            self.total_type_id += 1;
            id
        }
    }
}

pub struct InterpreterError {
    pub message: String,
}

impl InterpreterError {
    pub fn from_str(message: &str) -> InterpreterError {
        InterpreterError {
            message: message.to_string(),
        }
    }

    pub fn from_string(message: String) -> InterpreterError {
        InterpreterError { message: message }
    }
}

impl Interpreter {
    fn new(program: Box<Program>) -> Interpreter {
        let mut interpreter: Interpreter = Interpreter {
            program,
            top_env: Environment {
                variables: HashMap::new(),
                parent: None,
            },
            type_interner: TypeInterner::new(),
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
