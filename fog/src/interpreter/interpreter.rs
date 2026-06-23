use std::collections::HashMap;
use std::rc::Rc;

use crate::error::Span;
use crate::error::{FogError, FogResult};
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::nodes::Expr;
use crate::parser::nodes::Program;
use crate::parser::nodes::Statement::*;

pub struct Interpreter {
    pub program: Box<Program>,
    pub top_env: Environment,
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

        // the Type itself

        let var_type: Variable = Variable {
            name: "Type".to_string(),
            value: Some(Value::Type(Type::Type)),
            r#type: Type::Kind,
        };

        // primitive types

        let var_int32: Variable = Variable {
            name: "Int32".to_string(),
            value: Some(Value::Type(Type::Int32)),
            r#type: Type::Kind,
        };

        let var_float32: Variable = Variable {
            name: "Float32".to_string(),
            value: Some(Value::Type(Type::Float32)),
            r#type: Type::Kind,
        };

        let var_unit: Variable = Variable {
            name: "Unit".to_string(),
            value: Some(Value::Type(Type::Unit)),
            r#type: Type::Kind,
        };

        // function type

        let var_function: Variable = Variable {
            name: "->".to_string(),
            value: Some(Value::NativeFunction {
                param_type: Type::Type,
                return_type: Type::function(Type::Type, Type::Type),
                function: Rc::new(|arg: Value| match arg {
                    Value::Type(domain) => Ok(Value::NativeFunction {
                        param_type: Type::Type,
                        return_type: Type::Type,
                        function: Rc::new(move |arg: Value| match arg {
                            Value::Type(codomain) => Ok(Value::Type(Type::Function(
                                Box::new(domain.clone()),
                                Box::new(codomain.clone()),
                            ))),
                            _ => Err(FogError::runtime(
                                "expected a type argument".to_string(),
                                None,
                            )),
                        }),
                    }),
                    _ => Err(FogError::runtime(
                        "expected a type argument".to_string(),
                        None,
                    )),
                }),
            }),
            r#type: Type::function(Type::Type, Type::function(Type::Type, Type::Type)),
        };

        // builtin functions

        let var_plus_int32_int32: Variable = Variable {
            name: "_builtin_plus_Int32_Int32".to_string(),
            value: Some(Value::NativeFunction {
                param_type: Type::Int32,
                return_type: Type::Function(Box::new(Type::Int32), Box::new(Type::Int32)),
                function: Rc::new(|a: Value| match a {
                    Value::Int32(lhs) => Ok(Value::NativeFunction {
                        param_type: Type::Int32,
                        return_type: Type::Int32,
                        function: Rc::new(move |b: Value| match b {
                            Value::Int32(rhs) => Ok(Value::Int32(lhs + rhs)),
                            _ => Err(FogError::runtime(
                                "right operand is not an Int32".to_string(),
                                None,
                            )),
                        }),
                    }),
                    _ => Err(FogError::runtime(
                        "left operand is not an Int32".to_string(),
                        None,
                    )),
                }),
            }),
            r#type: Type::function(Type::Int32, Type::function(Type::Int32, Type::Int32)),
        };

        // built-in values

        let var_empty_tuple: Variable = Variable {
            name: "()".to_string(),
            value: Some(Value::EmptyTuple),
            r#type: Type::Kind,
        };

        vec![
            var_type,
            var_int32,
            var_float32,
            var_unit,
            var_function,
            var_plus_int32_int32,
            var_empty_tuple,
        ]
        .iter()
        .for_each(|var: &Variable| {
            interpreter
                .top_env
                .variables
                .insert(var.name.clone(), var.clone());
        });

        interpreter
    }

    pub fn run(program: Box<Program>) {
        let mut interpreter: Interpreter = Interpreter::new(program);
        let mut errors: Vec<FogError> = Vec::new();

        let top_env: &mut Environment = &mut interpreter.top_env;

        for stmt in &interpreter.program.statements {
            let result: Result<(), FogError> = match stmt {
                TypeAnnotation(name, expr, span) => annotate_type(name, expr, top_env, span),
                Declaration(name, expr, span) => declare(name, expr, top_env, span),
            };

            if let Err(error) = result {
                errors.push(error);
            }
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
            match error.span {
                Some(span) => println!(
                    "runtime error ({}:{}): {}",
                    span.line, span.column, error.message
                ),
                None => println!("runtime error: {}", error.message),
            }
        }
    }
}

fn annotate_type(name: &str, expr: &Expr, env: &mut Environment, span: &Span) -> FogResult<()> {
    let r#type: Rc<Type> = match eval_expr(&expr, env, span)? {
        Value::Type(r#type) => r#type.into(),
        _ => {
            return Err(FogError::runtime(
                "expression is not a type".to_string(),
                Some(span.clone()),
            ));
        }
    };

    (*env).annotate_type(name, (*r#type).clone(), span)
}

fn declare(name: &str, expr: &Expr, env: &mut Environment, span: &Span) -> FogResult<()> {
    let value: Value = eval_expr(expr, env, span)?;

    (*env).declare(name, value, span)
}
