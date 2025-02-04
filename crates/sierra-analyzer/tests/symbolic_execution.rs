use z3::{
    ast::{Ast, Int},
    Config, Context, SatResult,
};

use sierra_analyzer::sym_exec::sym_exec::SymbolicExecution;

#[test]
fn test_constraints() {
    let cfg = Config::new();
    let context = Context::new(&cfg);

    let mut sym_exec = SymbolicExecution::new(&context);

    // Declare the variables
    let v0 = Int::new_const(&context, "v0");
    let v1 = Int::new_const(&context, "v1");
    let v2 = Int::new_const(&context, "v2");

    // Create the constraints
    let constraint1 = v1._eq(&v0);
    let constraint2 = v2._eq(&(v1 + Int::from_i64(&context, 2)));
    let constraint3 = v2._eq(&Int::from_i64(&context, 0));

    // Load the constraints into the solver
    sym_exec.load_constraints(vec![&constraint1, &constraint2, &constraint3]);

    // Check if the constraints are satisfiable
    match sym_exec.check() {
        SatResult::Sat => {
            let model = sym_exec.solver.get_model().unwrap();
            let v0_value = model.eval(&v0, true).unwrap();
            assert_eq!(v0_value.as_i64().unwrap(), -2);
        }
        SatResult::Unsat => panic!("Constraints are unsatisfiable"),
        SatResult::Unknown => panic!("Satisfiability of constraints is unknown"),
    }
}

#[test]
fn test_add_constraint() {
    let cfg = Config::new();
    let context = Context::new(&cfg);

    let mut sym_exec = SymbolicExecution::new(&context);

    // Declare the variables
    let v0 = Int::new_const(&context, "v0");
    let v1 = Int::new_const(&context, "v1");
    let v2 = Int::new_const(&context, "v2");

    // Create the constraints
    let constraint1 = v1._eq(&v0);
    let constraint2 = v2._eq(&(v1 + Int::from_i64(&context, 2)));
    let constraint3 = v2._eq(&Int::from_i64(&context, 0));

    // Add the constraints one by one into the solver
    sym_exec.add_constraint(&constraint1);
    sym_exec.add_constraint(&constraint2);
    sym_exec.add_constraint(&constraint3);

    // Check if the constraints are satisfiable
    match sym_exec.check() {
        SatResult::Sat => {
            let model = sym_exec.solver.get_model().unwrap();
            let v0_value = model.eval(&v0, true).unwrap();
            assert_eq!(v0_value.as_i64().unwrap(), -2);
        }
        SatResult::Unsat => panic!("Constraints are unsatisfiable"),
        SatResult::Unknown => panic!("Satisfiability of constraints is unknown"),
    }
}
