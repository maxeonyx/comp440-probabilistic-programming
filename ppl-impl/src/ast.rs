#[derive(Debug)]
pub struct Program {
    pub definitions: Vec<Definition>,
    pub expression: Expression,
}

#[derive(Debug)]
pub struct Ident(pub String);

#[derive(Debug)]
pub struct Definition {
    pub ident: Ident,
    pub params: Vec<Ident>,
    pub body: Expression,
}

#[derive(Debug)]
pub struct Let {
    pub bindings: Vec<(Ident, Expression)>,
    pub body: Box<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    Variable(Ident),
    Let(Let),
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Negation(Box<Expression>),
    Sample(Box<Expression>),
    Observe(Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    FunctionApplication(Ident, Vec<Expression>),
    Vector(Vec<Expression>),
    HashMap(Vec<(Expression, Expression)>),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}
