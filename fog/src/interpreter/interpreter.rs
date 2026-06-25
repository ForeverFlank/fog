use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_expr;
use crate::interpreter::eval::eval_type_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::parsed_expr::ParsedExpr;
use crate::parser::parsed_expr::Program;
use crate::parser::parsed_expr::Statement::*;

pub struct Interpreter {
    pub program: Box<Program>,
    pub top_env: Environment,
}

impl Interpreter {
    fn new(program: Box<Program>) -> Interpreter {
        let mut interpreter: Interpreter = Interpreter {
            program,
            top_env: Environment::new(None),
        };

        // the Type itself
        let t_type: Type = Type::Type;

        // primitive types
        let t_int32: Type = Type::Int32;
        let t_float32: Type = Type::Float32;
        let t_unit: Type = Type::Product(Vec::new());

        // function type
        let t_function: Type = Type::Function(
            Box::new(Type::Type),
            Box::new(Type::Function(Box::new(Type::Type), Box::new(Type::Type))),
        );

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

        // insert into top level environment
        vec![t_type, t_int32, t_float32, t_unit, t_function]
            .iter()
            .for_each(|r#type: &Type| {
                interpreter
                    .top_env
                    .types
                    .insert(r#type.to_string(), r#type.clone());
            });

        vec![var_plus_int32_int32]
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
        all_vars.sort_by(|a: &Variable, b: &Variable| a.name.cmp(&b.name));

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

fn annotate_type(name: &str, expr: &ParsedExpr, env: &mut Environment, span: &Span) -> FogResult<()> {
    let r#type: Type = eval_type_expr(&expr, env)?;
    (*env).annotate_type(name, r#type, span)
}

fn declare(name: &String, expr: &ParsedExpr, env: &mut Environment, span: &Span) -> FogResult<()> {
    if env.variables.contains_key(name) {
        (*env).declare_value(name, eval_expr(expr, env, span)?, span)?;
        return Ok(());
    }

    if env.types.contains_key(name) {
        (*env).declare_type(name, eval_type_expr(expr, env)?, span)?;
        return Ok(());
    }

    Err(FogError::runtime(
        format!("unannotated variable `{}`", name),
        Some(span.clone()),
    ))
}
