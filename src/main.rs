mod tokenizer;

use std::fs;
use tokenizer::cursor::Cursor;

fn main() {
    let file_path = r"data/source1.txt";
    let result = fs::read_to_string(file_path);
    let Ok(txt) = result
        else {
            println!("File ni gevonne...");
            return;
        };
    parse(&txt);
}

fn parse(txt : &str) {
    println!("{0}", txt);
    let mut cur = Cursor::new(txt);
    let c = cur.peek();
    println!("first: {0}", c);
    let c = cur.peek_second();
    println!("second: {0}", c);
    let c = cur.peek(); //should be same
    println!("first: {0}", c);
    let c = cur.next(); // still same
    println!("first: {0}", c);
    let c = cur.peek(); // next char.
    println!("first: {0}", c);
    let tok = cur.next_token();
}
