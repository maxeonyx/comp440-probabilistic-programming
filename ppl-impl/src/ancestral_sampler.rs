use crate::{ast::Expression, types::Value};

enum FactorType {
    Sample,
    Observe,
}


// impl From<serde_json::Value> for Expression {
//     fn from(v: serde_json::Value) -> Self {
//         match v {
//             serde_json::Value::Null => Self::Null,
//             serde_json::Value::Bool(x) => Self::Boolean(x),
//             serde_json::Value::Number(x) => if x.is_f64() {
//                 Self::Float(x.as_f64().unwrap())
//             } else if x.is_i64() {
//                 Self::Integer(x.as_i64().unwrap())
//             } else {
//                 // Might overflow. Too bad.
//                 Self::Integer(x.as_u64().unwrap() as i64)
//             }
//             serde_json::Value::String(_) => { unimplemented!("String not implemented") }
//             serde_json::Value::Array(v) => {
//                 let first, rest
//                 match v.split_first() {
//                     Some(first, rest) => {
//                         match first {
//                             serde_json::Value::String(s) => {
//                                 Self::FunctionApplication(Ident(s))
//                             }
//                         }
//                     }
//                     None => Self::Vector(Vec::new()),
//                 }
//             },
//             serde_json::Value::Object(_) => { unimplemented!("Object/hashmap not implemented") }
//         }
//     }
// }

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(x) => Self::Boolean(x),
            serde_json::Value::Number(x) => if x.is_f64() {
                Self::Float(x.as_f64().unwrap())
            } else if x.is_i64() {
                Self::Integer(x.as_i64().unwrap())
            } else {
                // Might overflow. Too bad.
                Self::Integer(x.as_u64().unwrap() as i64)
            }
            serde_json::Value::String(_) => { unimplemented!("String not implemented") }
            serde_json::Value::Array(v) => Self::Vector(v.into_iter().map(Value::from).collect()),
            serde_json::Value::Object(_) => { unimplemented!("Object/hashmap not implemented") }
        }
    }
}

struct Pgm {
    variables: Vec<(usize, String)>,
    arcs: Vec<(usize, Vec<usize>)>,
    factors: Vec<(FactorType, Expression)>, 
    observations: Vec<(usize, Value)>,
    query: usize,
}
