use std::fs;
use macros::CastAny;
use math_parser::tokenizer::peeking_tokenizer::PeekingTokenizer;
use math_parser::parser::{CodeBlock, Parser};
use math_parser::parser::nodes::{BinExpr, ConstExpr, NodeData, print_nodes};
use math_parser::resolver::globals::Globals;
use math_parser::resolver::Resolver;
use math_parser::resolver::scope::Scope;
use math_parser::resolver::unit::Unit;
use math_parser::tokenizer::cursor::{Cursor, Number, Range};
use math_parser::tokenizer::indexing::FileIndex;
use math_parser::tokenizer::Token;
use math_parser::tokenizer::token_type::TokenType;
use math_parser::tokenizer::token_type::TokenType::Eot;

fn main() {
    // parse(&txt);
    // test_deref();
    test_resolver();
}

fn test_resolver() {
    // let text = "(20+10).m-31cm";
    let text = "abs(0-123)";
    let mut tok = PeekingTokenizer::new(text);
    let mut globals = Globals::new();
    globals.sources.push(&text);//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
    let scope = Scope::new(&globals);
    let code_block = CodeBlock::new(scope);

    //parse
    let mut parser = Parser::new(&mut tok, code_block);
    parser.parse(false);
    let mut code_block: CodeBlock = parser.into();
    println!("{}", text);
    for stmt in &code_block.statements {
        print_nodes(&stmt.node, 0);
    }

    //resolve
    let mut resolver = Resolver {
        scope: &mut code_block.scope,
        results: Vec::new(),
        errors: Vec::new(),
    };
    let results = resolver.resolve(&code_block.statements);

    let json_string = serde_json::to_string_pretty(&resolver).unwrap();
    println!("{}", json_string);
}

struct BytePos(i32);

impl Into<i32> for BytePos {
    fn into(self) -> i32 {
        self.0
    }
}

fn parse() {
    let file_path = r"data/source1.txt";
    let result = fs::read_to_string(file_path);
    let Ok(txt) = result
        else {
            println!("File ni gevonne...");
            return;
        };
    println!("{0}", txt);
    let txt = txt.as_str();

    let slice = &txt[53..56];
    let file_index = FileIndex::new(txt);

    println!("lines: {:?}", file_index.lines);
    println!("multibytes: {:?}", file_index.multi_byte_chars);

    let mut line_start: usize = 0;
    for line in &file_index.lines {
        let slice = &txt[line_start..*line];
        print!("line: {0}", slice); //TODO: strip endln chars?
        line_start = *line;
    }

    println!("slice: {0}", slice);

    let mut cur = Cursor::new(txt);

    loop {
        let tok = cur.next_token();
        match tok.kind {
            Eot => break,
            _ => {
                let (mut start_line, mut start_col) = file_index.get_line_and_column(tok.range.start);
                let (mut end_line, mut end_col) = file_index.get_line_and_column(tok.range.end);
                start_line += 1;
                start_col += 1;
                end_line += 1;
                end_col += 1;
                print!("[{0}:{1}, {2}:{3}] {4}", start_line, start_col, end_line, end_col,
                       &txt[tok.range.start..tok.range.end]);
                println!(" Token: {:?}", tok)
            }
        }
    }
    // println!("comment line: {0}", &txt[69..116]);
}



// unstable as of 12/2023
// const T_CONST_EXPR: TypeId = TypeId::of::<ConstExpr>();
// const T_BIN_EXPR: TypeId = TypeId::of::<BinExpr>();

fn test_deref() {
    let nod1 = BinExpr {
        expr1: Box::new(ConstExpr { value: Number{ significand: 12.0, exponent: 0, unit : Unit { range: None, id: "".to_string() }}, node_data: NodeData { has_errors: false, unit: Unit::none()} }),
        op: Token { kind: TokenType::Plus, range: Range { source_index: 0, start: 0, end: 0}},
        expr2: Box::new(ConstExpr { value: Number{ significand: 34.0, exponent: 0, unit : Unit { range: None, id: "".to_string() }} , node_data: NodeData { has_errors: false, unit: Unit::none()}}),
        node_data: NodeData { has_errors: false, unit : Unit::none()},
        implicit_mult: false
    };

    let mut nod1 = BinExpr {
        expr1: Box::new(nod1),
        op: Token { kind: TokenType::Plus, range: Range { source_index: 0, start: 0, end: 0 } },
        expr2: Box::new(ConstExpr { value: Number { significand: 56.0, exponent: 0 , unit : Unit { range: None, id: "".to_string() }}, node_data: NodeData { has_errors: false, unit: Unit::none() } }),
        node_data: NodeData { has_errors: false, unit: Unit::none() },
        implicit_mult: false
    };

    let nod1 = nod1.as_any_mut().downcast_mut::<BinExpr>().unwrap();

    // resolve_node(&mut nod1.expr1);
    // resolve_node(&mut nod1.expr2);
    println!("{0}", nod1.expr2.as_any().downcast_ref::<ConstExpr>().unwrap().value.significand);
}

#[cfg(test)]
mod test {
    use math_parser::parser::{CodeBlock, Parser};
    use math_parser::resolver::globals::Globals;
    use math_parser::resolver::Resolver;
    use math_parser::resolver::scope::Scope;
    use math_parser::resolver::value::Variant;
    use math_parser::tokenizer::peeking_tokenizer::PeekingTokenizer;

    #[test]
    fn test_simple_expr (){
        test_result("(1.3+2)*2", 6.6, "");
    }

    #[test]
    fn test_assign_expr () {
        test_result("a=1;b=2;c=a+b", 3.0, "");
    }

    #[test]
    fn test_function_calls () {
        test_result("abs(0-123)", 123.0, "");
    }

    #[test]
    fn test_units () {
        test_result("(10.3+3).m-300cm", 10.3, "m");
        test_result("1L", 1.0, "L");
        test_result("1L+100ml", 1.1, "L");
    }

    fn test_result(text: &str, expected_result: f64, unit: &str) {
        let mut tok = PeekingTokenizer::new(text);
        let mut globals = Globals::new();
        globals.sources.push(&text);//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
        let mut scope = Scope::new(&mut globals);
        let mut code_block = CodeBlock::new(scope);

        //parse
        let mut parser = Parser::new(&mut tok, code_block);
        parser.parse(false);
        let mut code_block: CodeBlock = parser.into();

        //resolve
        let mut resolver = Resolver {
            scope: &mut code_block.scope,
            results: Vec::new(),
            errors: Vec::new(),
        };
        let results = resolver.resolve(&code_block.statements);

        let value = resolver.results.last().expect("No result found.");
        let Variant::Number { number, .. } = &value.variant else {
            panic!("Result isn't a number.");
        };
        assert_eq!(number.to_double(), expected_result);
        assert_eq!(number.unit.id, unit);
    }
}