use core::num;
use std::{collections::HashMap, rc::Rc};

use crate::{ast, Distribution, EvalResult, Interpreter, RuntimeError, Value, ValueType};

enum ComparisonType {
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
    NotEqual,
}

fn assert_all_numeric_type(fn_name: &str, vals: &[Value]) -> Result<ValueType, RuntimeError> {
    // for 0 args we return Integer, which doesn't really make sense, but we should never call this with 0 args really.
    let mut all_t = ValueType::Integer;
    for val in vals {
        let this_el_type = match val {
            Value::Boolean(_) => ValueType::Boolean,
            Value::Float(_) => ValueType::Float,
            Value::Integer(_) => ValueType::Integer,
            Value::Distribution(_) => ValueType::Distribution,
            Value::Vector(_) => ValueType::Vector,
        };

        // float is contagious. if we ever see a float in a numeric operator the result is a float.
        if all_t == ValueType::Integer && this_el_type == ValueType::Integer {
            all_t = ValueType::Integer;
        } else if this_el_type == ValueType::Float {
            all_t = ValueType::Float;
        } else {
            return err!("Expected a numeric type, but found {}", this_el_type);
        }
    }

    return Ok(all_t);
}

impl Interpreter {
    pub fn dispatch_function(
        &mut self,
        name: &str,
        args: &[ast::Expression],
    ) -> Result<Value, RuntimeError> {
        // built-insd
        match name {
            "+" => return self.addition(args),
            "*" => return self.multiplication(args),
            "-" => return self.subtraction_or_negation(args),
            "/" => return self.division(args),

            "<" => return self.comparison(ComparisonType::Less, args),
            "<=" => return self.comparison(ComparisonType::LessEqual, args),
            "<>" => return self.comparison(ComparisonType::NotEqual, args),
            "=" => return self.comparison(ComparisonType::Equal, args),
            ">=" => return self.comparison(ComparisonType::GreaterEqual, args),
            ">" => return self.comparison(ComparisonType::Greater, args),

            "vector" => return self.vector(args),
            "hashmap" => return self.hashmap(args),
            "get" => return self.get(args),

            "log" => return self.log(args),
            "exp" => return self.exp(args),

            "bernoulli" => return self.bernoulli(args),
            "discrete" => return self.discrete(args),

            "normal" => return self.normal(args),
            "beta" => return self.beta(args),
            "poisson" => return self.poisson(args),
            _ => {}
        }

        // user-provided
        // look up function name in scope
        // easy for FOPPL

        err!("Could not find function {}", name)
    }

    fn addition(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() < 2 {
            return err!("Multiply must have at least 2 arguments.");
        }

        let mut sum_int = 0i64;
        let mut sum_float = 0f64;
        let mut all_int = true;
        for el in args {
            let val = self.eval(el)?;
            match val {
                Value::Float(v) => {
                    sum_float += v;
                    all_int = false;
                }
                Value::Integer(v) => {
                    sum_float += v as f64;
                    sum_int += v;
                }
                _ => return err!("Can't multiply types other than int and float."),
            }
        }

        Ok(if all_int {
            Value::Integer(sum_int)
        } else {
            Value::Float(sum_float)
        })
    }

    fn multiplication(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() < 2 {
            return err!("Multiply must have at least 2 arguments.");
        }

        let mut product_int = 1i64;
        let mut product_float = 1f64;
        let mut all_int = true;
        for el in args {
            let val = self.eval(el)?;
            match val {
                Value::Float(v) => {
                    product_float *= v;
                    all_int = false;
                }
                Value::Integer(v) => {
                    product_float *= v as f64;
                    product_int *= v;
                }
                _ => return err!("Can't multiply types other than int and float."),
            }
        }

        Ok(if all_int {
            Value::Integer(product_int)
        } else {
            Value::Float(product_float)
        })
    }

    fn subtraction_or_negation(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() == 1 {
            let vals = self.eval_all(args)?;
            // negation
            assert_all_numeric_type("negation", &vals)?;
            Ok(if let Value::Integer(a) = vals[0] {
                Value::Integer(-a)
            } else if let Value::Float(a) = vals[0] {
                Value::Float(-a)
            } else {
                unreachable!()
            })
        } else if args.len() == 2 {
            // subtraction
            let vals = self.eval_all(args)?;
            assert_all_numeric_type("subtraction", &vals)?;
            Ok(match (&vals[0], &vals[1]) {
                (Value::Integer(a), Value::Integer(b)) => Value::Integer(a - b),
                (Value::Float(a), Value::Integer(b)) => Value::Float(a - *b as f64),
                (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 - b),
                (Value::Float(a), Value::Float(b)) => Value::Float(a - b),
                _ => unreachable!(),
            })
        } else {
            err!("Too many arguments for subtraction or negation.")
        }
    }

    fn division(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() == 2 {
            // division
            let vals = self.eval_all(args)?;
            assert_all_numeric_type("division", &vals)?;
            Ok(match (&vals[0], &vals[1]) {
                (Value::Integer(a), Value::Integer(b)) => Value::Integer(a / b),
                (Value::Float(a), Value::Integer(b)) => Value::Float(a / *b as f64),
                (Value::Integer(a), Value::Float(b)) => Value::Float(*a as f64 / b),
                (Value::Float(a), Value::Float(b)) => Value::Float(a / b),
                _ => unreachable!(),
            })
        } else {
            err!("Too many arguments for subtraction or negation.")
        }
    }

    fn vector(&mut self, args: &[ast::Expression]) -> EvalResult {
        let vals = self.eval_all(args)?;
        Ok(Value::Vector(vals))
    }

    fn hashmap(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }

    fn get(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() != 2 {
            return err!("Get must have 2 arguments.");
        }

        let vals = self.eval_all(args)?;

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("First argument to get must be a vector."),
        };

        let index = match &vals[1] {
            Value::Integer(v) => *v,
            _ => return err!("Second argument to get must be an integer."),
        };

        if index as usize >= list.len() {
            return err!("Index out of bounds.");
        }

        Ok(list[index as usize].clone())
    }

    fn log(&mut self, args: &[ast::Expression]) -> EvalResult {
        if args.len() != 1 {
            return err!("log must have 1 argument.");
        }

        let vals = self.eval_all(args)?;

        assert_all_numeric_type("log", &vals)?;

        Ok(match vals[0] {
            Value::Integer(v) => Value::Float((v as f64).ln()),
            Value::Float(v) => Value::Float(v.ln()),
            _ => unreachable!(),
        })
    }

    fn exp(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }

    fn comparison(
        &mut self,
        comparison_type: ComparisonType,
        args: &[ast::Expression],
    ) -> EvalResult {
        fn compare<T: PartialOrd + PartialEq>(comparison_type: ComparisonType, a: T, b: T) -> bool {
            match comparison_type {
                ComparisonType::Less => a < b,
                ComparisonType::LessEqual => a <= b,
                ComparisonType::Greater => a > b,
                ComparisonType::GreaterEqual => a >= b,
                ComparisonType::Equal => a == b,
                ComparisonType::NotEqual => a != b,
            }
        }

        if args.len() == 2 {
            // division
            let vals = self.eval_all(args)?;
            assert_all_numeric_type("division", &vals)?;
            match (&vals[0], &vals[1]) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Boolean(compare(comparison_type, *a, *b)))
                }
                (Value::Float(a), Value::Integer(b)) => {
                    Ok(Value::Boolean(compare(comparison_type, *a, *b as f64)))
                }
                (Value::Integer(a), Value::Float(b)) => {
                    Ok(Value::Boolean(compare(comparison_type, *a as f64, *b)))
                }
                (Value::Float(a), Value::Float(b)) => {
                    Ok(Value::Boolean(compare(comparison_type, *a, *b)))
                }
                (Value::Vector(_a), Value::Vector(_b)) => match comparison_type {
                    _ => unimplemented!(),
                },
                _ => unreachable!(),
            }
        } else {
            err!("Too many arguments for subtraction or negation.")
        }
    }

    fn normal(&mut self, args: &[ast::Expression]) -> EvalResult {
        let vals = self.eval_all(args)?;
        if args.len() != 2 {
            return err!("Normal expects exactly two arguments.");
        }
        if let (Value::Float(mu), Value::Float(sigma)) = (&vals[0], &vals[1]) {
            let (mu, sigma) = (*mu, *sigma);
            let name = format!("normal({:?})", vals);
            let distribution = Value::Distribution(Distribution {
                sample: Rc::new(move || {
                    use rand::prelude::*;
                    use rand_distr::Normal;
                    let distr = Normal::new(mu, sigma).unwrap();
                    let mut rng = rand::thread_rng();
                    rng.sample::<f64, _>(distr)
                }),
                name,
            });
            Ok(distribution)
        } else {
            err!("Arguments to normal must be floats.")
        }
    }

    fn bernoulli(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }

    fn beta(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }

    fn poisson(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }

    fn discrete(&mut self, args: &[ast::Expression]) -> EvalResult {
        err!("Unimplemented")
    }
}
