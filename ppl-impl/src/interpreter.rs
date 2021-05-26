use crate::{
    ast::{self, Expression, Ident, Let, Program},
    types::{RuntimeError, Value},
};

use std::{collections::HashMap, convert::TryFrom, rc::Rc};

pub struct Binding {
    pub ident: String,
    pub val: Value,
}

pub struct Function {
    pub parameters: Vec<Ident>,
    pub body: Expression,
}

pub(crate) struct Interpreter {
    // TODO some mutable state for the observe side effects.
    // observe_state: u64,
    pub scope: Vec<Binding>,
    pub functions: HashMap<String, Rc<Function>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            functions: HashMap::new(),
            scope: Vec::new(),
        }
    }

    fn lookup_var(&self, var: &Ident) -> Option<Value> {
        for Binding { ident, val } in self.scope.iter().rev() {
            if *ident == var.0 {
                return Some(val.clone());
            }
        }

        None
    }

    pub fn eval_program(
        &mut self,
        program: Program,
        n_samples: usize,
    ) -> Result<Vec<Value>, RuntimeError> {
        for defn in program.definitions {
            let ast::Definition {
                ident,
                params,
                body,
            } = defn;
            let Ident(name) = ident;
            let function = Function {
                parameters: params,
                body: body,
            };
            self.functions.insert(name, Rc::new(function));
        }

        let expression = program.expression;
        (0..n_samples)
            .map(|_i| self.eval(&expression))
            .collect::<Result<Vec<Value>, RuntimeError>>()
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
                    Value::Distribution(d) => (d.sample)(),
                    _ => Err(RuntimeError::new(
                        "Sample must only be called on a Distribution value.".to_owned(),
                    )),
                }
            }
            Expression::FunctionApplication(ident, args) => {
                let vals = self.eval_all(args)?;
                self.dispatch_function(&ident.0, vals)
            }
            // Expression::Division(left, right) => {}
            // Expression::Subtraction(left, right) => {}
            // Expression::Negation(expr) => {}
            Expression::Observe(dist, val) => {
                // observe does nothing for now
                Ok(Value::Null)
            }
            Expression::ForEach(l) => {
                let ast::ForEach {
                    n_iters,
                    bindings,
                    body,
                } = l;
                let n_iters = *n_iters;
                // implements desugaring process from book
                let bindings = bindings
                    .iter()
                    .map(|(ident, expr)| {
                        let val = self.eval(expr)?;
                        let val = match val {
                            Value::Vector(v) => {
                                if v.len() != n_iters {
                                    return err!(
                                        "`foreach` binding vectors must have the specified length."
                                    );
                                } else {
                                    v
                                }
                            }
                            _ => return err!("`foreach` binding values must be vectors."),
                        };
                        Ok((ident.0.to_string(), val))
                    })
                    .collect::<Result<Vec<_>, RuntimeError>>()?;

                let mut return_vec = Vec::with_capacity(n_iters);
                for i in 0..n_iters {
                    let old_scope_count = self.scope.len();
                    self.scope
                        .extend(bindings.iter().map(|(name, vec)| Binding {
                            ident: name.clone(),
                            val: vec[i].clone(),
                        }));
                    let vals = self.eval_all(body);
                    self.scope.truncate(old_scope_count);
                    let mut vals = vals?;
                    return_vec.push(vals.pop().unwrap());
                }

                Ok(Value::Vector(return_vec))
            }
            Expression::Loop(l) => {
                let ast::Loop {
                    n_iters,
                    accumulator,
                    fn_name,
                    params,
                } = l;
                // implements desugaring process from book
                let mut accumulator = self.eval(accumulator)?;
                let params = self.eval_all(params)?;
                for i in 0..*n_iters {
                    let idx = i64::try_from(i).map_err(|_| {
                        RuntimeError::new("Loop index overflow. Shouldn't be possible.".to_string())
                    })?;
                    let mut args = vec![Value::Integer(idx), accumulator];
                    args.extend(params.clone().into_iter());
                    accumulator = self.dispatch_function(&fn_name.0, args)?;
                }

                Ok(accumulator)
            }
            // Expression::If(comp, true_branch, false_branch) => {}
            Expression::Vector(elements) => Ok(Value::Vector(self.eval_all(elements)?)),
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
