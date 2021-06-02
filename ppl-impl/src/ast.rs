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
    pub body: Vec<Expression>,
}

#[derive(Debug)]
pub struct ForEach {
    pub n_iters: usize,
    pub bindings: Vec<(Ident, Expression)>,
    pub body: Vec<Expression>,
}

#[derive(Debug)]
pub struct Loop {
    pub n_iters: usize,
    pub accumulator: Box<Expression>,
    pub fn_name: Ident,
    pub params: Vec<Expression>,
}

#[derive(Debug)]
pub enum Expression {
    Variable(Ident),
    Let(Let),
    Sample(Box<Expression>),
    Observe(Box<Expression>, Box<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    FunctionApplication(Ident, Vec<Expression>),
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Vector(Vec<Expression>),
    ForEach(ForEach),
    Loop(Loop),
    Null,
}
