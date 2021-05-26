use core::num;
use std::{collections::HashMap, rc::Rc};

use crate::{
    ast,
    interpreter::{Binding, Interpreter},
    types::{Distribution, RuntimeError, Value, ValueType},
    EvalResult,
};

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
            return err!(
                "{} expected a numeric type, but found {}",
                fn_name,
                this_el_type
            );
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
            "first" => return self.first(vals),
            "second" => return self.second(vals),
            "last" => return self.last(vals),
            "rest" => return self.rest(vals),
            "append" => return self.append(vals),

            "mat-transpose" => return self.matrix_transpose(vals),
            "mat-mul" => return self.matrix_multiply(vals),
            "mat-add" => return self.matrix_addition(vals),

            "log" => return self.log(vals),
            "exp" => return self.exp(vals),
            "sqrt" => return self.sqrt(vals),

            // "bernoulli" => return self.bernoulli(vals),
            "discrete" => return self.discrete(vals),

            "normal" => return self.normal(vals),
            // "beta" => return self.beta(vals),
            // "poisson" => return self.poisson(vals),
            _ => {}
        }

        // user-provided
        // easy for FOPPL, no scope needed
        // look up function name in self.functions

        if self.functions.contains_key(name) {
            let function = self.functions.get(name).unwrap();
            let old_scope_count = self.scope.len();

            if vals.len() != function.parameters.len() {
                return err!(
                    "{} expected {} arguments but got {}",
                    name,
                    function.parameters.len(),
                    vals.len()
                );
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

            return Ok(val);
        }
        err!("Could not find function `{}`", name)
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
        Ok(Value::Vector(
            vals.iter().map(|v| v.clone()).collect::<Vec<_>>(),
        ))
    }

    fn hashmap(&mut self, vals: Vec<Value>) -> EvalResult {
        err!("Unimplemented")
    }

    fn get(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("`get` must have 2 arguments.");
        }

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("First argument to `get` must be a vector."),
        };

        let index = match &vals[1] {
            Value::Integer(v) => *v,
            _ => return err!("Second argument to `get` must be an integer."),
        };

        if index as usize >= list.len() {
            return err!("Index out of bounds.");
        }

        Ok(list[index as usize].clone())
    }

    fn first(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`first` must have 1 argument.");
        }

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `first` must be a vector."),
        };

        if list.len() < 1 {
            return err!("Index out of bounds.");
        }

        Ok(list[0].clone())
    }

    fn second(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`second` must have 1 argument.");
        }

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `second` must be a vector."),
        };

        if list.len() < 2 {
            return err!("Index out of bounds.");
        }

        Ok(list[1].clone())
    }

    fn last(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`last` must have exactly 1 argument.");
        }

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `last` must be a vector."),
        };

        if list.len() < 1 {
            return err!("Index out of bounds.");
        }

        Ok(list[list.len() - 1].clone())
    }

    fn rest(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`rest` must have exactly 1 argument.");
        }

        let mut list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to 'rest' must be a vector."),
        };

        if list.len() < 1 {
            return err!("Index out of bounds.");
        }

        Ok(Value::Vector(list[1..].iter().cloned().collect()))
    }

    fn matrix_transpose(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`mat-transpose` must have exactly 1 argument.");
        }

        let list = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-transpose` must be a vector of vectors."),
        };

        if list.len() < 1 {
            return err!("The vector given to `mat-transpose` must have at least one element.");
        }

        let first_el = match &list[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-transpose` must be a vector of vectors."),
        };

        if first_el.len() < 1 {
            return err!(
                "All sub-vectors given to `mat-transpose` must have at least one element."
            );
        }

        let unwrapped = list
            .iter()
            .map(|v| match v {
                Value::Vector(v) => {
                    if v.len() != first_el.len() {
                        return err!("`mat-transpose` needs a 2D, rectangular vector-of-vectors.");
                    }
                    Ok(v.clone())
                }
                _ => return err!("`mat-transpose` needs a 2D, rectangular vector-of-vectors."),
            })
            .collect::<Result<Vec<Vec<Value>>, RuntimeError>>()?;

        let mut transposed = Vec::with_capacity(first_el.len());
        for i in 0..first_el.len() {
            let mut row = Vec::with_capacity(list.len());
            for j in 0..list.len() {
                row.push(unwrapped[j][i].clone());
            }
            transposed.push(Value::Vector(row));
        }

        Ok(Value::Vector(transposed))
    }

    fn matrix_multiply(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("`mat-multiply` must have exactly 2 arguments.");
        }

        let mat1 = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument 1 to `mat-multiply` must be a vector of vectors."),
        };

        let mat2 = match &vals[1] {
            Value::Vector(v) => v,
            _ => return err!("Argument 2 to `mat-multiply` must be a vector of vectors."),
        };

        if mat1.len() < 1 {
            return err!("The vector given to `mat-multiply` must have at least one element.");
        }

        let mat1_first_el = match &mat1[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-multiply` must be a vector of vectors."),
        };

        if mat1_first_el.len() < 1 {
            return err!("All sub-vectors given to `mat-multiply` must have at least one element.");
        }

        let mat2_first_el = match &mat2[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-multiply` must be a vector of vectors."),
        };

        if mat2_first_el.len() < 1 {
            return err!("All sub-vectors given to `mat-multiply` must have at least one element.");
        }

        let unwrapped_mat1 = mat1
            .iter()
            .map(|v| match v {
                Value::Vector(v) => {
                    if v.len() != mat1_first_el.len() {
                        return err!("First arg to `mat-multiply` had uneven length rows.");
                    }
                    assert_all_numeric_type("mat-multiply", v)?;
                    let v = v
                        .iter()
                        .map(|v| match v {
                            Value::Float(f) => *f,
                            Value::Integer(i) => *i as f64,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<f64>>();
                    Ok(v)
                }
                _ => return err!("First arg to `mat-multiply` had non-vector elements."),
            })
            .collect::<Result<Vec<Vec<f64>>, RuntimeError>>()?;

        let unwrapped_mat2 = mat2
            .iter()
            .map(|v| match v {
                Value::Vector(v) => {
                    if v.len() != mat2_first_el.len() {
                        return err!("Second arg to `mat-multiply` had uneven length rows.");
                    }
                    assert_all_numeric_type("mat-multiply", v)?;
                    let v = v
                        .iter()
                        .map(|v| match v {
                            Value::Float(f) => *f,
                            Value::Integer(i) => *i as f64,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<f64>>();
                    Ok(v)
                }
                _ => return err!("Second arg to `mat-multiply` had non-vector elements."),
            })
            .collect::<Result<Vec<Vec<f64>>, RuntimeError>>()?;

        let mat1_nrows = unwrapped_mat1.len();
        let mat1_ncols = mat1_first_el.len();
        let mat2_nrows = unwrapped_mat2.len();
        let mat2_ncols = mat2_first_el.len();

        if mat1_ncols != mat2_nrows {
            return err!("`mat-multiply` needs matrices with matching inner dimensions.");
        }

        let shared_dim = mat1_ncols;

        let mut product = Vec::with_capacity(mat1_nrows);
        for i in 0..mat1_nrows {
            let mut row = Vec::with_capacity(mat2_ncols);
            for j in 0..mat2_ncols {
                let mut sum = 0f64;
                for k in 0..shared_dim {
                    let v1 = unwrapped_mat1[i][k];
                    let v2 = unwrapped_mat2[k][j];

                    sum += v1 * v2;
                }
                row.push(Value::Float(sum));
            }
            product.push(Value::Vector(row));
        }

        Ok(Value::Vector(product))
    }

    fn matrix_addition(&mut self, vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("`mat-multiply` must have exactly 2 arguments.");
        }

        let mat1 = match &vals[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument 1 to `mat-multiply` must be a vector of vectors."),
        };

        let mat2 = match &vals[1] {
            Value::Vector(v) => v,
            _ => return err!("Argument 2 to `mat-multiply` must be a vector of vectors."),
        };

        if mat1.len() < 1 {
            return err!("The vector given to `mat-multiply` must have at least one element.");
        }

        let mat1_first_el = match &mat1[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-multiply` must be a vector of vectors."),
        };

        if mat1_first_el.len() < 1 {
            return err!("All sub-vectors given to `mat-multiply` must have at least one element.");
        }

        let mat2_first_el = match &mat2[0] {
            Value::Vector(v) => v,
            _ => return err!("Argument to `mat-multiply` must be a vector of vectors."),
        };

        if mat2_first_el.len() < 1 {
            return err!("All sub-vectors given to `mat-multiply` must have at least one element.");
        }

        let unwrapped_mat1 = mat1
            .iter()
            .map(|v| match v {
                Value::Vector(v) => {
                    if v.len() != mat1_first_el.len() {
                        return err!("First arg to `mat-multiply` had uneven length rows.");
                    }
                    assert_all_numeric_type("mat-multiply", v)?;
                    let v = v
                        .iter()
                        .map(|v| match v {
                            Value::Float(f) => *f,
                            Value::Integer(i) => *i as f64,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<f64>>();
                    Ok(v)
                }
                _ => return err!("First arg to `mat-multiply` had non-vector elements."),
            })
            .collect::<Result<Vec<Vec<f64>>, RuntimeError>>()?;

        let unwrapped_mat2 = mat2
            .iter()
            .map(|v| match v {
                Value::Vector(v) => {
                    if v.len() != mat2_first_el.len() {
                        return err!("Second arg to `mat-multiply` had uneven length rows.");
                    }
                    assert_all_numeric_type("mat-multiply", v)?;
                    let v = v
                        .iter()
                        .map(|v| match v {
                            Value::Float(f) => *f,
                            Value::Integer(i) => *i as f64,
                            _ => unreachable!(),
                        })
                        .collect::<Vec<f64>>();
                    Ok(v)
                }
                _ => return err!("Second arg to `mat-multiply` had non-vector elements."),
            })
            .collect::<Result<Vec<Vec<f64>>, RuntimeError>>()?;

        let mat1_nrows = unwrapped_mat1.len();
        let mat1_ncols = mat1_first_el.len();
        let mat2_nrows = unwrapped_mat2.len();
        let mat2_ncols = mat2_first_el.len();

        let compatible_dimensions =
            (mat1_nrows == 1 || mat2_nrows == 1 || mat2_nrows == mat2_nrows)
                && (mat1_ncols == 1 || mat2_ncols == 1 || mat2_ncols == mat1_ncols);

        if !compatible_dimensions {
            return err!("arguments to `mat-add` have incompatible dimensions.");
        }

        let n_rows = mat1_nrows.max(mat2_nrows);
        let n_cols = mat1_ncols.max(mat2_ncols);

        let mut sum_mat = Vec::with_capacity(n_rows);
        for i in 0..n_rows {
            let mut row = Vec::with_capacity(n_cols);
            let m1i = if mat1_nrows > 1 { i } else { 1 };
            let m2i = if mat2_nrows > 1 { i } else { 1 };
            for j in 0..n_cols {
                let m1j = if mat1_ncols > 1 { j } else { 1 };
                let m2j = if mat2_ncols > 1 { j } else { 1 };
                row.push(Value::Float(
                    unwrapped_mat1[m1i][m1j] + unwrapped_mat2[m2i][m2j],
                ));
            }
            sum_mat.push(Value::Vector(row));
        }

        Ok(Value::Vector(sum_mat))
    }

    fn append(&mut self, mut vals: Vec<Value>) -> EvalResult {
        if vals.len() != 2 {
            return err!("`append` must have exactly 2 arguments.");
        }

        let el = vals.pop().unwrap();

        let mut vec = match vals.pop().unwrap() {
            Value::Vector(v) => v,
            _ => return err!("First argument to `append` must be a vector."),
        };

        vec.push(el);

        Ok(Value::Vector(vec))
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
        if vals.len() != 1 {
            return err!("log must have 1 argument.");
        }

        assert_all_numeric_type("exp", &vals)?;

        Ok(match vals[0] {
            Value::Integer(v) => Value::Float((v as f64).exp()),
            Value::Float(v) => Value::Float(v.exp()),
            _ => unreachable!(),
        })
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

    fn comparison(&mut self, comparison_type: ComparisonType, vals: Vec<Value>) -> EvalResult {
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
                let distr = match Normal::new(mu, sigma) {
                    Ok(dist) => dist,
                    Err(_) => return err!("Error creating discrete distribution."),
                };
                let mut rng = rand::thread_rng();
                Ok(Value::Float(rng.sample::<f64, _>(distr)))
            }),
            name,
        });
        Ok(distribution)
    }

    fn discrete(&mut self, mut vals: Vec<Value>) -> EvalResult {
        if vals.len() != 1 {
            return err!("`discrete` expects exactly 1 argument, a vector of numbers.");
        }
        let vals = match vals.pop().unwrap() {
            Value::Vector(v) => v,
            _ => return err!("`discrete` expects exactly 1 argument, a vector of numbers."),
        };
        assert_all_numeric_type("discrete", &vals)?;
        let name = format!("discrete({:?})", &vals);
        let weights = vals
            .into_iter()
            .map(|v| match v {
                Value::Float(f) => f,
                Value::Integer(i) => i as f64,
                _ => unreachable!(),
            })
            .collect::<Vec<f64>>();
        let distribution = Value::Distribution(Distribution {
            sample: Rc::new(move || {
                use rand::prelude::*;
                use rand_distr::WeightedIndex;
                let distr = match WeightedIndex::new(&weights) {
                    Ok(w) => w,
                    Err(_) => return err!("Error creating `discrete` distribution."),
                };
                let mut rng = rand::thread_rng();
                let val = rng.sample::<usize, _>(distr);
                Ok(Value::Integer(val as i64))
            }),
            name,
        });
        Ok(distribution)
    }
}
