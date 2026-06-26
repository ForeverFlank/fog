use std::rc::Rc;

use crate::error::FogError;
use crate::error::FogResult;
use crate::error::Span;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_type_annotation_expr;
use crate::interpreter::eval::eval_type_definition_expr;
use crate::interpreter::eval::eval_value_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;
use crate::parser::resolved_expr::ResolvedExpr;
use crate::parser::resolved_expr::ResolvedStatement;
use crate::parser::resolved_expr::ResolvedStatement::Declaration;
use crate::parser::resolved_expr::ResolvedStatement::TypeAnnotation;

pub struct Interpreter {
    pub statements: Vec<ResolvedStatement>,
    pub top_env: Environment,
}

impl Interpreter {
    fn new(statements: Vec<ResolvedStatement>) -> Interpreter {
        let mut interpreter = Interpreter {
            statements,
            top_env: Environment::new(None),
        };

        // the Type itself
        let t_type = Type::Type;

        // primitive types
        let t_int32 = Type::Int32;
        let t_float32 = Type::Float32;
        let t_unit = Type::Product(Vec::new());

        // function type
        let t_function = Type::Function(
            Box::new(Type::Type),
            Box::new(Type::Function(Box::new(Type::Type), Box::new(Type::Type))),
        );

        // builtin functions
        let var_plus_int32_int32 = Variable {
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
            .for_each(|r#type| {
                interpreter
                    .top_env
                    .types
                    .insert(r#type.to_string(), r#type.clone());
            });

        vec![var_plus_int32_int32].iter().for_each(|var| {
            interpreter
                .top_env
                .variables
                .insert(var.name.clone(), var.clone());
        });

        interpreter
    }

    pub fn run(statements: Vec<ResolvedStatement>) {
        let mut interpreter = Interpreter::new(statements);
        let mut errors = Vec::new();

        let top_env = &mut interpreter.top_env;

        for stmt in &interpreter.statements {
            let result: Result<(), FogError> = match stmt {
                TypeAnnotation { name, expr, span } => eval_type_annotation_expr(expr, top_env)
                    .and_then(|r#type| top_env.annotate_type(name, r#type, span)),
                Declaration { name, expr, span } => Self::declare(name, expr, top_env, span),
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

    fn declare(
        name: &String,
        expr: &ResolvedExpr,
        env: &mut Environment,
        span: &Span,
    ) -> FogResult<()> {
        if env.variables.contains_key(name) {
            if let Type::Type = env.variables[name].r#type {
                env.variables.remove(name);
                let r#type = eval_type_definition_expr(expr, env)?;
                env.declare_type(name, r#type.clone(), span)?;
                return Ok(());
            }

            env.declare_value(name, eval_value_expr(expr, env, span)?, span)?;
            return Ok(());
        }

        if env.types.contains_key(name) {
            let r#type = eval_type_definition_expr(expr, env)?;
            env.declare_type(name, r#type.clone(), span)?;
            return Ok(());
        }

        Err(FogError::runtime(
            format!("unannotated variable `{}`", name),
            Some(span.clone()),
        ))
    }
}
