use crate::types::{RuntimeError, Value};


pub trait Distribution: std::fmt::Debug {
    fn sample(&self) -> Result<Value, RuntimeError>;
    fn log_pdf(&self, val: &Value) -> Result<f64, RuntimeError>;
    fn name(&self) -> &'static str;
}

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

    fn log_pdf(&self, val: &Value) -> Result<f64, RuntimeError> {

        let val = match val {
            Value::Float(f) => *f,
            Value::Integer(i) => *i as f64,
            _ => return err!("Normal distribution can only eval density of a float."),
        };

        let log_density = -(self.sigma * std::f64::consts::TAU.sqrt()).ln() + -(1f64 / 2f64) * ((val - self.mu).powi(2) / self.sigma);

        Ok(log_density)
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

    fn log_pdf(&self, val: &Value) -> Result<f64, RuntimeError> {
        use probability::distribution::{Categorical, Discrete};
        let d = Categorical::new(&self.weights);

        let val = match val {
            Value::Integer(v) if *v >= 0 => *v as usize,
            _ => return err!("Discrete distribution can only eval density of a positive integer."),
        };

        Ok(d.mass(val).ln())
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

pub struct Gamma {
    pub alpha: f64,
    pub beta: f64,
}

impl Distribution for Gamma {
    fn sample(&self) -> Result<Value, RuntimeError> {
        use rand::prelude::*;
        use rand_distr::Gamma;
        let distr = match Gamma::new(self.alpha, self.beta) {
            Ok(w) => w,
            Err(_) => return err!("Error creating `gamma` distribution."),
        };
        let mut rng = rand::thread_rng();
        let val = rng.sample(distr);
        Ok(Value::Float(val))
    }

    fn log_pdf(&self, val: &Value) -> Result<f64, RuntimeError> {
        use probability::distribution::{Continuous, Gamma};
        let d = Gamma::new(self.alpha, self.beta);

        let val = val.clone().try_into_numeric(
            "`gamma` can only evaluate the density of a floating point number.",
        )?;
        Ok(d.density(val).ln())
    }

    fn name(&self) -> &'static str {
        "gamma"
    }
}

impl std::fmt::Debug for Gamma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(...)", self.name())
    }
}

pub struct Dirichlet {
    pub parameters: Vec<f64>,
}

impl Distribution for Dirichlet {
    fn sample(&self) -> Result<Value, RuntimeError> {
        use rand::prelude::*;
        use rand_distr::Dirichlet;
        let distr = match Dirichlet::new(&self.parameters) {
            Ok(w) => w,
            Err(_) => return err!("Error creating `gamma` distribution."),
        };
        let mut rng = rand::thread_rng();
        let vals = rng.sample(distr);
        Ok(Value::Vector(
            vals.into_iter().map(|x| Value::Float(x)).collect(),
        ))
    }

    fn log_pdf(&self, vals: &Value) -> Result<f64, RuntimeError> {
        unimplemented!("Evaluating the density of `dirichlet` is not implemented.")
    }

    fn name(&self) -> &'static str {
        "dirichlet"
    }
}

impl std::fmt::Debug for Dirichlet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(...)", self.name())
    }
}
