mod ast;

use std::{borrow::Borrow, collections::BinaryHeap, error::Error, ops::Deref};

use ast::{Expression, Ident, Let};
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);

fn main() -> Result<(), Box<dyn Error>> {
    let text = include_str!("../examples/addition.ppl");
    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(text)?;
    println!("{:?}", ast);

    let val = Interpreter::new().eval(&ast.expression);

    println!("{:?}", val);

    Ok(())
}

fn run(program: &ast::Program) {}

#[derive(Clone, Debug)]
pub enum Distribution {
    Normal(f64, f64),
}

impl Distribution {
    fn sample(&self, params: Vec<f64>) -> f64 {
        use Distribution::*;
        match self {
            Normal(mu, sigma) => 0.0f64,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Distribution(Box<Distribution>),
    Vector(Vec<Value>),
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

struct Binding {
    ident: String,
    val: Value,
}

struct Interpreter {
    // TODO some mutable state for the observe side effects.
    // observe_state: u64,
    scope: Vec<Binding>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter { scope: Vec::new() }
    }

    fn lookup_var(&self, var: &Ident) -> Option<Value> {
        for Binding { ident, val } in self.scope.iter().rev() {
            if *ident == var.0 {
                return Some(val.clone());
            }
        }

        None
    }

    fn eval(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Variable(var) => match self.lookup_var(var) {
                Some(val) => Ok(val),
                None => Err(RuntimeError::new(format!(
                    "Variable {} not defined.",
                    var.0
                ))),
            },
            Expression::Let(Let { bindings, body }) => {
                let old_scope_count = self.scope.len();
                for (ident, expr) in bindings {
                    let val = match self.eval(expr) {
                        Ok(v) => v,
                        Err(e) => {
                            self.scope.truncate(old_scope_count);
                            return Err(e);
                        }
                    };
                    let binding = Binding {
                        ident: ident.0.clone(),
                        val,
                    };
                    self.scope.push(binding);
                }

                let val = self.eval(body);
                self.scope.truncate(old_scope_count);
                val
            }
            Expression::Addition(left, right) => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;
                if let (Value::Float(left), Value::Float(right)) = (&left_val, &right_val) {
                    return Ok(Value::Float(left + right));
                } else if let (Value::Integer(left), Value::Integer(right)) =
                    (&left_val, &right_val)
                {
                    return Ok(Value::Integer(left + right));
                }
                Err(RuntimeError::new(
                    "Must use addition on floats or integers only.".to_owned(),
                ))
            }
            Expression::Integer(val) => Ok(Value::Integer(*val)),
            _ => unimplemented!(),
            /*
            Expression::Multiplication(left, right) => un
            Expression::Division(left, right) => {}
            Expression::Subtraction(left, right) => {}
            Expression::Negation(expr) => {}
            Expression::Sample(expr) => {}
            Expression::Observe(dist, val) => {}
            Expression::If(comp, true_branch, false_branch) => {}
            Expression::FunctionApplication(ident, params) => {}
            Expression::Vector(elements) => {}
            Expression::HashMap(pairs) => {}
            Expression::Boolean(val) => {}
            Expression::Float(val) => {}
            */
        }
    }
}
