mod ast;

macro_rules! err {
    ($fstr:literal $(, $e:expr)*) => {
        Err(RuntimeError::new(format!($fstr, $($e,)*)))
    };
}

type EvalResult = Result<Value, RuntimeError>;

mod functions;

use core::fmt;
use std::rc::Rc;

use ast::{Expression, Ident, Let};
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);

fn main() {
    let mut args = std::env::args();
    let _executable_name = args.next();
    let filename = match args.next() {
        Some(s) => s,
        None => {
            eprintln!("Must provide a filename argument.");
            return;
        }
    };
    let text = match std::fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading \"{}\": {}", filename, e);
            return;
        }
    };

    let parser = grammar::ProgramParser::new();
    let ast = match parser.parse(&text) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Error parsing: {}", e);
            return;
        }
    };
    println!("{:?}", ast);

    let val = Interpreter::new().eval(&ast.expression);

    println!("{:?}", val);
}

#[derive(Clone)]
pub struct Distribution {
    sample: Rc<dyn Fn() -> f64>,
    name: String,
}

impl std::fmt::Debug for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Distribution")
            .field("name", &self.name)
            .finish()
    }
}

#[derive(PartialEq, Debug)]
pub enum ValueType {
    Float,
    Integer,
    Boolean,
    Distribution,
    Vector,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Distribution(Distribution),
    Vector(Vec<Value>),
}

#[derive(Debug, Clone)]
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
            Expression::Integer(val) => Ok(Value::Integer(*val)),
            Expression::Float(val) => Ok(Value::Float(*val)),
            Expression::Sample(expr) => {
                let val = self.eval(expr)?;
                match val {
                    Value::Distribution(d) => Ok(Value::Float((d.sample)())),
                    _ => Err(RuntimeError::new(
                        "Sample must only be called on a Distribution value.".to_owned(),
                    )),
                }
            }
            Expression::FunctionApplication(ident, args) => self.dispatch_function(&ident.0, &args),
            // Expression::Division(left, right) => {}
            // Expression::Subtraction(left, right) => {}
            // Expression::Negation(expr) => {}
            // Expression::Observe(dist, val) => {}
            // Expression::If(comp, true_branch, false_branch) => {}
            // Expression::Vector(elements) => {}
            // Expression::HashMap(pairs) => {}
            // Expression::Boolean(val) => {}
            x => Err(RuntimeError::new(format!("Unimplemented: {:?}", x))),
        }
    }

    fn eval_all(&mut self, args: &[ast::Expression]) -> Result<Vec<Value>, RuntimeError> {
        let mut vals = Vec::with_capacity(args.len());
        for arg in args {
            vals.push(self.eval(arg)?);
        }
        Ok(vals)
    }
}
