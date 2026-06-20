use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::nodes::Expr;
use crate::ast::nodes::Program;
use crate::ast::nodes::Statement::*;
use crate::error::{FogError, FogResult};
use crate::interpreter::environment::Environment;
use crate::interpreter::eval::eval_expr;
use crate::interpreter::r#type::Type;
use crate::interpreter::value::Value;
use crate::interpreter::variable::Variable;

pub struct Interpreter {
    pub program: Box<Program>,
    pub top_env: Environment,
    // pub type_interner: TypeInterner,
}

// pub struct TypeInterner {
//     pub type_map: HashMap<Type, TypeId>,
//     pub total_type_id: TypeId,
// }

// pub type TypeId = i32;

// impl TypeInterner {
//     pub fn new() -> TypeInterner {
//         TypeInterner {
//             type_map: HashMap::new(),
//             total_type_id: 0,
//         }
//     }

//     pub fn get_or_add_type(&mut self, r#type: Type) -> TypeId {
//         if let Some(id) = self.type_map.get(&r#type) {
//             *id
//         } else {
//             let id: i32 = self.total_type_id;
//             self.type_map.insert(r#type, id);
//             self.total_type_id += 1;
//             id
//         }
//     }
// }

impl Interpreter {
    fn new(program: Box<Program>) -> Interpreter {
        let mut interpreter: Interpreter = Interpreter {
            program,
            top_env: Environment {
                variables: HashMap::new(),
                parent: None,
            },
            // type_interner: TypeInterner::new(),
        };

        let var_type: Variable = Variable {
            name: "Type".to_string(),
            value: Some(Value::Type(Type::Type)),
            r#type: Type::Kind,
        };

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
        let var_plus_int_int: Variable = Variable {
            name: "_builtin_plus_int_int".to_string(),
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

        vec![var_type, var_int32, var_float32, var_plus_int_int]
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
            let result = match stmt {
                TypeAnnotation(name, expr) => annotate_type(name, expr, top_env),
                Declaration(name, expr) => declare(name, expr, top_env),
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
            println!("error: {}", error.message)
        }
    }
}

fn annotate_type(name: &str, expr: &Expr, env: &mut Environment) -> FogResult<()> {
    let r#type: Rc<Type> = match eval_expr(&expr, env)? {
        Value::Type(r#type) => r#type.into(),
        _ => {
            return Err(FogError::runtime(
                "expression is not a type".to_string(),
                None,
            ));
        }
    };

    (*env).annotate_type(name, (*r#type).clone())
}

fn declare(name: &str, expr: &Expr, env: &mut Environment) -> FogResult<()> {
    let value: Value = eval_expr(expr, env)?;

    (*env).declare(name, value)
}
