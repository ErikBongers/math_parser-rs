mod tokenizer;
mod parser;
mod resolver;

use std::any::{Any, TypeId};
use std::fs;
use cast_any::CastAny;
use tokenizer::cursor::Cursor;
use crate::parser::nodes::{BinExpr, ConstExpr, Node, NodeData};
use crate::tokenizer::cursor::{Number, Range};
use crate::tokenizer::indexing::FileIndex;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::Token;
use crate::tokenizer::token_type::TokenType;
use crate::tokenizer::token_type::TokenType::Eot;

fn main() {
    // parse(&txt);
    test_deref();
    test_parser();
}

fn test_parser() {
    let txt = "2 + 3 * 4";
    let mut tok = PeekingTokenizer::new(txt);
    let mut code_block = crate::parser::CodeBlock { statements: Vec::new()};
    let mut parser = crate::parser::Parser::new(&mut tok, &mut code_block);
    parser.parse();
    let stmt = parser.code_block.statements.first().expect("There should be a statement here.");
    parser::print_nodes(&stmt.node, 0);
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
        expr1: Box::new(ConstExpr { value: Number{ significand: 12.0, exponent: 0}, node_data: NodeData { error: 0, unit: 0} }),
        op: Token { kind: TokenType::Plus, range: Range { source_index: 0, start: 0, end: 0}},
        expr2: Box::new(ConstExpr { value: Number{ significand: 34.0, exponent: 0} , node_data: NodeData { error: 0, unit: 0}}),
        node_data: NodeData { error: 0, unit : 0},
    };

    let mut nod1 = BinExpr {
        expr1: Box::new(nod1),
        op: Token { kind: TokenType::Plus, range: Range { source_index: 0, start: 0, end: 0 } },
        expr2: Box::new(ConstExpr { value: Number { significand: 56.0, exponent: 0 }, node_data: NodeData { error: 0, unit: 0 } }),
        node_data: NodeData { error: 0, unit: 0 },
    };

    let nod1 = nod1.as_any_mut().downcast_mut::<BinExpr>().unwrap();

    // resolve_node(&mut nod1.expr1);
    // resolve_node(&mut nod1.expr2);
    println!("{0}", nod1.expr2.as_any().downcast_ref::<ConstExpr>().unwrap().value.significand);
}

