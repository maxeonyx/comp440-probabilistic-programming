use std::{fmt, rc::Rc};

#[derive(Clone)]
pub struct Distribution {
    pub sample: Rc<dyn Fn() -> Result<Value, RuntimeError>>,
    pub name: String,
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
    Distribution(Distribution),
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
