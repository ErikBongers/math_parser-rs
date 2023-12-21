mod tokenizer;

use cast_any_derive::CastAny;
use std::any::{Any, TypeId};
use std::fs;
use cast_any::CastAny;
use tokenizer::cursor::Cursor;
use crate::tokenizer::indexing::FileIndex;
use crate::tokenizer::token_type::TokenType::Eot;

fn main() {
    // parse(&txt);
    test_deref();
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


//--- TEST Dereferencing

pub trait Node: CastAny {

}

#[derive(CastAny)]
pub struct BinExpr {
    pub expr1: Box<dyn Node>,
    pub expr2: Box<dyn Node>,
}

impl Node for BinExpr {}

#[derive(CastAny)]
pub struct ConstExpr {
    pub value: i32,
}

impl Node for ConstExpr {}

// unstable as of 12/2023
// const T_CONST_EXPR: TypeId = TypeId::of::<ConstExpr>();
// const T_BIN_EXPR: TypeId = TypeId::of::<BinExpr>();

fn test_deref() {
    let mut nod1 = BinExpr {
        expr1: Box::new(ConstExpr { value: 12 }),
        expr2: Box::new(ConstExpr { value: 34 }) };

    let mut nod1 = BinExpr {
        expr1: Box::new(nod1),
        expr2: Box::new(ConstExpr { value: 56 }) };

    let nod1 = nod1.as_any_mut().downcast_mut::<BinExpr>().unwrap();

    resolve_node(&mut nod1.expr1);
    resolve_node(&mut nod1.expr2);
    println!("{0}", nod1.expr2.as_any().downcast_ref::<ConstExpr>().unwrap().value);
}

fn resolve_node(expr: &mut Box<dyn Node>) {
    match expr.as_any().type_id() {
        t if TypeId::of::<ConstExpr>() == t => {
            let expr = expr.as_any_mut().downcast_mut::<ConstExpr>().unwrap();
            resolve_const_expr(expr);
            println!("value: {0}", expr.as_any().downcast_ref::<ConstExpr>().unwrap().value)
        },
        t if TypeId::of::<BinExpr>() == t => {
            // resolve_bin_expr(expr.as_any_mut().downcast_ref::<BinExpr>().unwrap());
            let bin_expr = expr.as_any_mut().downcast_mut::<BinExpr>().unwrap();
            resolve_node(&mut bin_expr.expr1);
            resolve_node(&mut bin_expr.expr2);
        },
        _ => {
            println!("It's a dunno...");
        }
    }
}

fn resolve_const_expr(const_expr: &mut ConstExpr) {
    println!("It's a ConstExpr!");
    const_expr.value = -const_expr.value;
}

fn resolve_bin_expr(_bin_expr: &BinExpr) {
    println!("It's a BinExpr!")
}