use std::{fmt, rc::Rc};

pub trait Distribution: std::fmt::Debug {
    fn sample(&self) -> Result<Value, RuntimeError>;
    fn pdf(&self, val: Value) -> Result<f64, RuntimeError>;
    fn name(&self) -> &'static str;
}

#[derive(PartialEq, Debug)]
pub enum ValueType {
    Float,
    Integer,
    Boolean,
    Distribution,
    Vector,
    Null,
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
    Distribution(Rc<dyn Distribution>),
    Vector(Vec<Value>),
    Null,
}

impl Value {
    pub fn get_type(&self) -> ValueType {
        match self {
            Self::Float(_) => ValueType::Float,
            Self::Integer(_) => ValueType::Integer,
            Self::Boolean(_) => ValueType::Boolean,
            Self::Distribution(_) => ValueType::Distribution,
            Self::Vector(_) => ValueType::Vector,
            Self::Null => ValueType::Null,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
