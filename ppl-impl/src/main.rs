mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

fn main() {
    let text = include_str!("../examples/addition.ppl");
    let parser = grammar::ProgramParser::new();
    let ast = parser.parse(text);
    println!("{:?}", ast);
}
