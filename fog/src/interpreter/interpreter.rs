use crate::anf::anf::ANFStatement;
use crate::core::get_interpreter_variables;
use crate::error::FogResult;
use crate::interpreter::environment::Environment;
use crate::interpreter::eval_value::eval_scope;
use crate::interpreter::variable::ValueVariable;
use crate::runtime_error;

fn create_top_env() -> Environment<'static> {
    let mut env = Environment::new(None);

    for var in get_interpreter_variables() {
        env.variables.insert(var.name.clone(), var);
    }

    env
}

pub fn interpret(anfs: &Vec<ANFStatement>) -> FogResult<()> {
    // check for final operand in the top-level statement
    // will replace this when we have actual main function
    // for anf in anfs {
    //     if let ANFStatement::Value(_) = anf {
    //         return Err(runtime_error!(
    //             // Some(*span), // TODO span
    //             None,
    //             "cannot have final operand as a top-level statement"
    //         ));
    //     }
    // }

    let mut top_env = create_top_env();
    eval_scope(anfs, &mut top_env)?;

    // let mut all_vars: Vec<ValueVariable> = top_env.variables.values().cloned().collect();
    // all_vars.sort_by(|a, b| a.name.cmp(&b.name));

    // println!();
    // for var in all_vars {
    //     println!(
    //         "{} = {}",
    //         var.name,
    //         match &*var.value.borrow() {
    //             Some(value) => value.to_string(),
    //             None => "[undefined]".to_string(),
    //         }
    //     );
    // }
    // println!();

    Ok(())
}
