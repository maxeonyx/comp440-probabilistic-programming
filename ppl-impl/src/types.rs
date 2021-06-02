use std::{
    error::Error,
    fmt::{self, Display},
    rc::Rc,
};

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

    pub fn try_into_usize(self, message: &str) -> Result<usize, RuntimeError> {
        match self {
            Value::Integer(x) if x > 0 => Ok(x as usize),
            _ => err!("{}", message.to_owned()),
        }
    }

    pub fn try_into_integer(self, message: &str) -> Result<i64, RuntimeError> {
        match self {
            Value::Integer(x) => Ok(x),
            _ => err!("{}", message.to_owned()),
        }
    }

    pub fn try_into_float(self, message: &str) -> Result<f64, RuntimeError> {
        match self {
            Value::Float(x) => Ok(x),
            _ => err!("{}", message.to_owned()),
        }
    }

    pub fn try_into_numeric(self, message: &str) -> Result<f64, RuntimeError> {
        match self {
            Value::Float(x) => Ok(x),
            Value::Integer(x) => Ok(x as f64),
            _ => err!("{}", message.to_owned()),
        }
    }

    pub fn try_into_vector(self, message: &str) -> Result<Vec<Value>, RuntimeError> {
        match self {
            Value::Vector(x) => Ok(x),
            _ => err!("{}", message.to_owned()),
        }
    }
}

pub trait ValueImpls {
    fn try_into_numeric(self, message: &str) -> Result<Vec<f64>, RuntimeError>;
    fn try_into_one(self, message: &str) -> Result<Value, RuntimeError>;
}

impl ValueImpls for Vec<Value> {
    fn try_into_numeric(self, message: &str) -> Result<Vec<f64>, RuntimeError> {
        self.into_iter()
            .map(|v| match v {
                Value::Float(x) => Ok(x),
                Value::Integer(x) => Ok(x as f64),
                _ => err!("{}", message.to_owned()),
            })
            .collect::<Result<Vec<f64>, RuntimeError>>()
    }

    fn try_into_one(mut self, message: &str) -> Result<Value, RuntimeError> {
        if self.len() != 1 {
            return err!("{}", message.to_owned());
        }
        Ok(self.pop().unwrap())
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    source: Option<Box<dyn Error>>,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        Self {
            message,
            source: None,
        }
    }
    fn wrap<T: Error + 'static>(e: T) -> Self {
        Self {
            message: String::new(),
            source: Some(Box::new(e)),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime Error: {}", self.message)?;
        if let Some(e) = self.source.as_deref() {
            write!(f, "Caused by:")?;
            Display::fmt(e, f)?;
        };
        Ok(())
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref()
    }

    // fn type_id(&self, _: private::Internal) -> std::any::TypeId
    // where
    //     Self: 'static,
    // {
    //     std::any::TypeId::of::<Self>()
    // }

    // fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
    //     None
    // }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source.as_deref()
    }
}
