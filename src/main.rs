use std::fs;
use math_parser::{parse_1_file, parse_2_files};

fn main() {
    // parse_one_file();
    parse_two_files();
}

#[allow(unused)]
fn parse_two_files() {
    let file_path = r"data/source1.txt";
    let file_path2 = r"data/source2.txt";
    let result = fs::read_to_string(file_path);
    let Ok(text1) = result
        else {
            println!("File1 ni gevonne...");
            return;
        };
    let result = fs::read_to_string(file_path2);
    let Ok(text2) = result
        else {
            println!("File2 ni gevonne...");
            return;
        };
    println!("{}", parse_2_files(text1, text2));
}

#[allow(unused)]
fn parse_one_file() {
    let file_path = r"data/source1.txt";

    let result = fs::read_to_string(file_path);
    let Ok(text) = result
        else {
            println!("File ni gevonne...");
            return;
        };
    println!("{0}", text);
    println!("{}", parse_1_file(text));
}


#[cfg(test)]
mod test;