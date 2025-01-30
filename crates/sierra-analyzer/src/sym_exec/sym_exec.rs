use std::collections::HashSet;
use std::str::FromStr;

use z3::{
    ast::{Ast, Bool, Int},
    Config, Context, SatResult, Solver,
};

use cairo_lang_sierra::program::GenStatement;

use crate::decompiler::function::{Function, SierraStatement};
use crate::decompiler::libfuncs_patterns::{
    ADDITION_REGEX, CONST_REGEXES, DUP_REGEX, IS_ZERO_REGEX, MULTIPLICATION_REGEX,
    SUBSTRACTION_REGEX,
};
use crate::{extract_parameters, parse_element_name_with_fallback};

/// Converts a SierraStatement to a Z3 constraint, or returns None if not applicable
pub fn sierra_statement_to_constraint<'ctx>(
    statement: &SierraStatement,
    context: &'ctx Context,
    declared_libfuncs_names: Vec<String>,
) -> Option<Bool<'ctx>> {
    match &statement.statement {
        GenStatement::Invocation(invocation) => {
            let libfunc_id_str =
                parse_element_name_with_fallback!(invocation.libfunc_id, declared_libfuncs_names);
            let parameters = extract_parameters!(invocation.args);
            let assigned_variables = invocation
                .branches
                .first()
                .map(|branch| extract_parameters!(&branch.results))
                .unwrap_or_else(Vec::new);

            handle_invocation(context, &libfunc_id_str, &parameters, &assigned_variables)
        }
        _ => None,
    }
}

/// Handles an invocation by trying to match it to known patterns.
fn handle_invocation<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    parameters: &[String],
    assigned_variables: &[String],
) -> Option<Bool<'ctx>> {
    handle_duplication(context, libfunc_id_str, assigned_variables)
        .or_else(|| handle_constant_assignment(context, libfunc_id_str, assigned_variables))
        .or_else(|| handle_is_zero(context, libfunc_id_str, parameters))
        .or_else(|| {
            handle_arithmetic_operations(context, libfunc_id_str, parameters, assigned_variables)
        })
}

/// Handles variable duplication in Sierra statements
fn handle_duplication<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    assigned_variables: &[String],
) -> Option<Bool<'ctx>> {
    if DUP_REGEX.is_match(libfunc_id_str) {
        let first_var_z3 = Int::new_const(context, assigned_variables[0].clone());
        let second_var_z3 = Int::new_const(context, assigned_variables[1].clone());
        return Some(second_var_z3._eq(&first_var_z3).into());
    }
    None
}

/// Handles constant assignment in Sierra statements
fn handle_constant_assignment<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    assigned_variables: &[String],
) -> Option<Bool<'ctx>> {
    for regex in CONST_REGEXES.iter() {
        if let Some(captures) = regex.captures(libfunc_id_str) {
            if let Some(const_value) = captures.name("const") {
                let const_value_str = const_value.as_str();
                if let Ok(const_value_u64) = u64::from_str(const_value_str) {
                    if !assigned_variables.is_empty() {
                        let assigned_var_z3 = Int::new_const(context, &*assigned_variables[0]);
                        let const_value_z3 = Int::from_u64(context, const_value_u64);
                        return Some(assigned_var_z3._eq(&const_value_z3).into());
                    }
                }
            }
        }
    }
    None
}

/// Handles zero check in Sierra statements
fn handle_is_zero<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    parameters: &[String],
) -> Option<Bool<'ctx>> {
    if IS_ZERO_REGEX.is_match(libfunc_id_str) {
        let operand = Int::new_const(context, parameters[0].clone());
        let constraint = operand._eq(&Int::from_i64(context, 0));
        return Some(constraint);
    }
    None
}

/// Handles arithmetic operations in Sierra statements
fn handle_arithmetic_operations<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    parameters: &[String],
    assigned_variables: &[String],
) -> Option<Bool<'ctx>> {
    let operator = if ADDITION_REGEX
        .iter()
        .any(|regex| regex.is_match(libfunc_id_str))
    {
        "+"
    } else if SUBSTRACTION_REGEX
        .iter()
        .any(|regex| regex.is_match(libfunc_id_str))
    {
        "-"
    } else if MULTIPLICATION_REGEX
        .iter()
        .any(|regex| regex.is_match(libfunc_id_str))
    {
        "*"
    } else {
        return None;
    };

    let assigned_variable = Int::new_const(context, assigned_variables[0].clone());
    let first_operand = Int::new_const(context, parameters[0].clone());
    let second_operand = Int::new_const(context, parameters[1].clone());

    let constraint = match operator {
        "+" => assigned_variable._eq(&(first_operand + second_operand)),
        "-" => assigned_variable._eq(&(first_operand - second_operand)),
        "*" => assigned_variable._eq(&(first_operand * second_operand)),
        _ => return None,
    };

    Some(constraint)
}

/// Generates test cases for a single function
pub fn generate_test_cases_for_function(
    function: &mut Function,
    declared_libfuncs_names: Vec<String>,
) -> String {
    let mut result = String::new();
    let mut unique_results = HashSet::new();

    let felt252_arguments: Vec<(String, String)> = function
        .arguments
        .iter()
        .filter(|(_, arg_type)| arg_type == "felt252")
        .map(|(arg_name, arg_type)| (arg_name.clone(), arg_type.clone()))
        .collect();

    // Skip the function if there are no felt252 arguments
    if felt252_arguments.is_empty() {
        return result;
    }

    // Generate the function CFG
    function.create_cfg();

    let function_paths = function.cfg.as_ref().unwrap().paths();

    for path in &function_paths {
        // Create a new symbolic execution engine for the function
        let cfg = Config::new();
        let context = Context::new(&cfg);

        // Create Z3 variables for each felt252 argument
        let z3_variables: Vec<Int> = felt252_arguments
            .iter()
            .map(|(arg_name, _)| Int::new_const(&context, &**arg_name))
            .collect();

        // Create a solver
        let mut symbolic_execution = SymbolicExecution::new(&context);

        let mut zero_constraints = Vec::new();
        let mut other_constraints = Vec::new();

        // Convert Sierra statements to z3 constraints
        for basic_block in path {
            for statement in &basic_block.statements {
                // Convert SierraStatement to a Z3 constraint and add to solver
                if let Some(constraint) = sierra_statement_to_constraint(
                    &statement,
                    &context,
                    declared_libfuncs_names.clone(),
                ) {
                    symbolic_execution.add_constraint(&constraint);

                    // Identify if it's a zero check and store the variable for non-zero testing
                    if let GenStatement::Invocation(invocation) = &statement.statement {
                        let libfunc_id_str = parse_element_name_with_fallback!(
                            invocation.libfunc_id,
                            declared_libfuncs_names
                        );

                        if IS_ZERO_REGEX.is_match(&libfunc_id_str) {
                            let operand_name = format!("v{}", invocation.args[0].id.to_string());
                            let operand = Int::new_const(&context, operand_name.clone());
                            zero_constraints.push((operand, constraint));
                        } else {
                            // Store other constraints for reuse
                            other_constraints.push(constraint);
                        }
                    }
                }
            }

            // Check if the constraints are satisfiable (value == 0)
            generate_zero_test_cases(
                &mut result,
                &mut unique_results,
                &symbolic_execution,
                &felt252_arguments,
                &z3_variables,
            );

            // Now generate test cases where the value is not equal to 0
            generate_non_zero_test_cases(
                &mut result,
                &mut unique_results,
                &context,
                &felt252_arguments,
                &z3_variables,
                &zero_constraints,
                &other_constraints,
            );
        }
    }

    result.trim_end().to_string()
}

/// Generates test cases where the constraints are satisfiable (value == 0).
fn generate_zero_test_cases(
    result: &mut String,
    unique_results: &mut HashSet<String>,
    symbolic_execution: &SymbolicExecution,
    felt252_arguments: &[(String, String)],
    z3_variables: &[Int],
) {
    // Check if the constraints are satisfiable
    if symbolic_execution.check() == SatResult::Sat {
        // Get the model from the solver
        if let Some(model) = symbolic_execution.solver.get_model() {
            // Evaluate the variables and format the results
            let values: Vec<String> = felt252_arguments
                .iter()
                .zip(z3_variables.iter())
                .map(|((arg_name, _), var)| {
                    format!(
                        "{}: {}",
                        arg_name,
                        model.eval(var, true).unwrap().to_string()
                    )
                })
                .collect();
            let values_str = format!("{}", values.join(", "));
            // Add the result to the unique results set and the result string
            if unique_results.insert(values_str.clone()) {
                result.push_str(&format!("{}\n", values_str));
            }
        }
    }
}

/// Generates test cases where the value is not equal to 0.
fn generate_non_zero_test_cases(
    result: &mut String,
    unique_results: &mut HashSet<String>,
    context: &Context,
    felt252_arguments: &[(String, String)],
    z3_variables: &[Int],
    zero_constraints: &[(Int, Bool)],
    other_constraints: &[Bool],
) {
    for (operand, _) in zero_constraints {
        // Create a fresh solver for the non-zero case
        let non_zero_solver = Solver::new(context);

        // Re-apply all other constraints except the zero-equality one
        for constraint in other_constraints {
            non_zero_solver.assert(constraint);
        }

        // Add a constraint to force the operand to be not equal to 0
        non_zero_solver.assert(&operand._eq(&Int::from_i64(context, 0)).not());

        // Check if the constraints are satisfiable
        if non_zero_solver.check() == SatResult::Sat {
            // Get the model from the solver
            if let Some(model) = non_zero_solver.get_model() {
                // Evaluate the variables and format the results
                let values: Vec<String> = felt252_arguments
                    .iter()
                    .zip(z3_variables.iter())
                    .map(|((arg_name, _), var)| {
                        format!(
                            "{}: {}",
                            arg_name,
                            model.eval(var, true).unwrap().to_string()
                        )
                    })
                    .collect();
                let values_str = format!("{}", values.join(", "));
                // Add the result to the unique results set and the result string
                if unique_results.insert(values_str.clone()) {
                    result.push_str(&format!("{}\n", values_str));
                }
            }
        }
    }
}

/// Parses the result of generate_test_cases_for_function and returns a vector of vectors of integer inputs
/// TODO : Reverse the logic and generate formatted testcases from the integers vectors
pub fn get_integers_inputs(test_cases: &str) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    let unique_results: HashSet<String> = test_cases.lines().map(|line| line.to_string()).collect();

    for line in unique_results {
        let mut line_inputs = Vec::new();
        let parts: Vec<&str> = line.split(", ").collect();
        for part in parts {
            let key_value: Vec<&str> = part.split(": ").collect();
            if key_value.len() == 2 {
                if let Ok(value) = i64::from_str(key_value[1]) {
                    line_inputs.push(value);
                }
            }
        }
        result.push(line_inputs);
    }

    result
}

/// A struct that represents a symbolic execution solver
#[derive(Debug)]
pub struct SymbolicExecution<'a> {
    pub solver: Solver<'a>,
}

impl<'a> SymbolicExecution<'a> {
    /// Creates a new instance of `SymbolicExecution`
    pub fn new(context: &'a Context) -> Self {
        let solver = Solver::new(context);

        SymbolicExecution { solver }
    }

    /// Loads constraints into the Z3 solver
    pub fn load_constraints(&mut self, constraints: Vec<&Bool<'a>>) {
        for constraint in constraints {
            self.solver.assert(constraint);
        }
    }

    /// Adds a single constraint into the Z3 solver
    pub fn add_constraint(&mut self, constraint: &Bool<'a>) {
        self.solver.assert(constraint);
    }

    /// Checks if the current set of constraints is satisfiable
    pub fn check(&self) -> z3::SatResult {
        self.solver.check()
    }
}
