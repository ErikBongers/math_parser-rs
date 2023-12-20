mod tokenizer;

use std::fs;
use tokenizer::cursor::Cursor;
use crate::tokenizer::indexing::FileIndex;
use crate::tokenizer::token_type::TokenType::Eot;

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

struct BytePos(i32);

impl Into<i32> for BytePos {
    fn into(self) -> i32 {
        self.0
    }
}

fn parse(txt : &str) {
    println!("{0}", txt);

    let slice = &txt[53..56];
    let file_index = FileIndex::new(txt);

    println!("lines: {:?}", file_index.lines);
    println!("multibytes: {:?}", file_index.multi_byte_chars);

    let mut line_start :usize = 0;
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
                start_line+=1;
                start_col+=1;
                end_line+=1;
                end_col+=1;
                print!("[{0}:{1}, {2}:{3}] {4}", start_line, start_col, end_line, end_col,
                    &txt[tok.range.start..tok.range.end]);
                println!(" Token: {:?}", tok)
            }
        }
    }
    // println!("comment line: {0}", &txt[69..116]);
}
