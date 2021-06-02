use crate::types::{Distribution, RuntimeError, Value};

pub struct Normal {
    pub mu: f64,
    pub sigma: f64,
}

impl Distribution for Normal {
    fn sample(&self) -> Result<Value, RuntimeError> {
        use rand::prelude::*;
        use rand_distr::Normal;
        let distr = match Normal::new(self.mu, self.sigma) {
            Ok(dist) => dist,
            Err(_) => return err!("Error creating discrete distribution."),
        };
        let mut rng = rand::thread_rng();
        Ok(Value::Float(rng.sample::<f64, _>(distr)))
    }

    fn pdf(&self, val: Value) -> Result<f64, RuntimeError> {
        use probability::distribution::{Continuous, Gaussian};
        let d = Gaussian::new(self.mu, self.sigma);

        let val = match val {
            Value::Float(f) => f,
            Value::Integer(i) => i as f64,
            _ => return err!("Normal distribution can only eval density of a float."),
        };

        Ok(d.density(val))
    }

    fn name(&self) -> &'static str {
        "normal"
    }
}

impl std::fmt::Debug for Normal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(...)", self.name())
    }
}

pub struct Discrete {
    pub weights: Vec<f64>,
}

impl Distribution for Discrete {
    fn sample(&self) -> Result<Value, RuntimeError> {
        use rand::prelude::*;
        use rand_distr::WeightedIndex;
        let distr = match WeightedIndex::new(&self.weights) {
            Ok(w) => w,
            Err(_) => return err!("Error creating `discrete` distribution."),
        };
        let mut rng = rand::thread_rng();
        let val = rng.sample::<usize, _>(distr);
        Ok(Value::Integer(val as i64))
    }

    fn pdf(&self, val: Value) -> Result<f64, RuntimeError> {
        use probability::distribution::{Categorical, Discrete};
        let d = Categorical::new(&self.weights);

        let val = match val {
            Value::Integer(v) if v >= 0 => v as usize,
            _ => return err!("Normal distribution can only eval density of a positive integer."),
        };

        Ok(d.mass(val))
    }

    fn name(&self) -> &'static str {
        "discrete"
    }
}

impl std::fmt::Debug for Discrete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(...)", self.name())
    }
}
