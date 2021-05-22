use core::num;
use std::{collections::HashMap, rc::Rc};

use crate::{EvalResult, ast, interpreter::{Binding, Interpreter}, types::{Distribution, RuntimeError, Value, ValueType}};


enum ComparisonType {
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
    NotEqual,
}



fn assert_all_numeric_type(fn_name: &str, vals: &[Value]) -> Result<ValueType, RuntimeError> {
    // for 0 vals we return Integer, which doesn't really make sense, but we should never call this with 0 vals really.
    let mut all_t = ValueType::Integer;
    for val in vals {
        let this_el_type = val.get_type();

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
        vals: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // built-insd
        match name {
            "+" => return self.addition(vals),
            "*" => return self.multiplication(vals),
            "-" => return self.subtraction_or_negation(vals),
            "/" => return self.division(vals),

            "<" => return self.comparison(ComparisonType::Less, vals),
            "<=" => return self.comparison(ComparisonType::LessEqual, vals),
            "<>" => return self.comparison(ComparisonType::NotEqual, vals),
            "=" => return self.comparison(ComparisonType::Equal, vals),
            ">=" => return self.comparison(ComparisonType::GreaterEqual, vals),
            ">" => return self.comparison(ComparisonType::Greater, vals),

            "vector" => return self.vector(vals),
            "hashmap" => return self.hashmap(vals),
            "get" => return self.get(vals),

            "log" => return self.log(vals),
            "exp" => return self.exp(vals),
            "sqrt" => return self.sqrt(vals),

            "bernoulli" => return self.bernoulli(vals),
            "discrete" => return self.discrete(vals),

            "normal" => return self.normal(vals),
            "beta" => return self.beta(vals),
            "poisson" => return self.poisson(vals),
            _ => {}
        }

        // user-provided
        // look up function name in self.functions
        
        if self.functions.contains_key(name) {
            let function = self.functions.get(name).unwrap();
            let old_scope_count = self.scope.len();
            
            if vals.len() != function.parameters.len() {
                return err!("{} expected {} arguments but got {}", name, function.parameters.len(), vals.len());
            }

            for (ident, val) in function.parameters.iter().zip(vals.into_iter()) {
                let binding = Binding {
                    ident: ident.0.clone(),
                    val,
                };
                self.scope.push(binding);
            }

            let val = self.eval(&function.clone().body)?;
            self.scope.truncate(old_scope_count);
            
            return Ok(val)
        }
        // easy for FOPPL

        err!("Could not find function {}", name)
    }

    fn addition(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() < 2 {
            return err!("Multiply must have at least 2 arguments.");
        }

        let mut sum_int = 0i64;
        let mut sum_float = 0f64;
        let mut all_int = true;
        for val in vals {
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

    fn multiplication(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() < 2 {
            return err!("Multiply must have at least 2 arguments.");
        }

        let mut product_int = 1i64;
        let mut product_float = 1f64;
        let mut all_int = true;
        for val in vals {
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

    fn subtraction_or_negation(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() == 1 {
            // negation
            assert_all_numeric_type("negation", &vals)?;
            Ok(if let Value::Integer(a) = vals[0] {
                Value::Integer(-a)
            } else if let Value::Float(a) = vals[0] {
                Value::Float(-a)
            } else {
                unreachable!()
            })
        } else if vals.len() == 2 {
            // subtraction
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

    fn division(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() == 2 {
            // division
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

    fn vector(&mut self, vals: Vec<Value>) -> EvalResult {
        Ok(Value::Vector(vals.iter().map(|v| v.clone()).collect::<Vec<_>>()))
    }

    fn hashmap(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn get(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("Get must have 2 arguments.");
        }

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

    fn log(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("log must have 1 argument.");
        }

        assert_all_numeric_type("log", &vals)?;

        Ok(match vals[0] {
            Value::Integer(v) => Value::Float((v as f64).ln()),
            Value::Float(v) => Value::Float(v.ln()),
            _ => unreachable!(),
        })
    }

    fn exp(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn comparison(
        &mut self,
        comparison_type: ComparisonType,
        vals: Vec<Value>,
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

        if vals.len() != 2 {
            return err!("Comparison must have exactly two arguments.");
        }

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
                _ => unimplemented!("Vector comparison not implemented."),
            },
            _ => unimplemented!("Comparison for this type combination not implemented."),
        }
    }

    fn normal(&mut self, mut vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("Normal expects exactly two arguments.");
        }
        assert_all_numeric_type("normal", &vals)?;
        let (sigma, mu) = (vals.pop().unwrap(), vals.pop().unwrap());
        let mu = match mu {
            Value::Float(v) => v,
            Value::Integer(v) => v as f64,
            _ => unreachable!(),
        };
        let sigma = match sigma {
            Value::Float(v) => v,
            Value::Integer(v) => v as f64,
            _ => unreachable!(),
        };

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
    }

    fn sqrt(&mut self, mut vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("Sqrt expects exactly one argument.");
        }

        assert_all_numeric_type("sqrt", &vals)?;

        let val = vals.pop().unwrap();

        let val = match val {
            Value::Float(v) => Value::Float(v.sqrt()),
            Value::Integer(v) => Value::Float((v as f64).sqrt()),
            _ => unreachable!(),
        };

        Ok(val)
    }

    fn bernoulli(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn beta(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn poisson(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn discrete(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }
}
