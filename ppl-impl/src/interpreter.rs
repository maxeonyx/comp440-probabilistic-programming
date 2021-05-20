use crate::{
    ast::{self, Expression, Ident, Let},
    types::{RuntimeError, Value},
};

struct Binding {
    ident: String,
    val: Value,
}

pub(crate) struct Interpreter {
    // TODO some mutable state for the observe side effects.
    // observe_state: u64,
    scope: Vec<Binding>,
}

impl Interpreter {
    pub fn new() -> Self {
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

    pub fn eval(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Variable(var) => match self.lookup_var(var) {
                Some(val) => Ok(val),
                None => Err(RuntimeError::new(format!(
                    "Variable {} not defined.",
                    var.0
                ))),
            },
            Expression::Let(Let { bindings, body }) => {
                if bindings.len() < 1 {
                    return err!("Let must have at least one binding.");
                }

                if body.len() < 1 {
                    return err!("Let must have a body.");
                }

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

                let vals = self.eval_all(body);
                self.scope.truncate(old_scope_count);
                let mut vals = vals?;
                Ok(vals.pop().unwrap())
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
            Expression::Observe(dist, val) => {
                // observe does nothing for now
                Ok(Value::Null)
            }
            // Expression::If(comp, true_branch, false_branch) => {}
            // Expression::Vector(elements) => {}
            // Expression::HashMap(pairs) => {}
            // Expression::Boolean(val) => {}
            x => Err(RuntimeError::new(format!("Unimplemented: {:?}", x))),
        }
    }

    pub fn eval_all(&mut self, args: &[ast::Expression]) -> Result<Vec<Value>, RuntimeError> {
        let mut vals = Vec::with_capacity(args.len());
        for arg in args {
            vals.push(self.eval(arg)?);
        }
        Ok(vals)
    }
}
