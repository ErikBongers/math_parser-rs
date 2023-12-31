use std::fs;
use math_parser::parse_and_print_nodes;

fn main() {
    test_resolver();
}

fn test_resolver() {
    let file_path = r"data/source1.txt";
    let result = fs::read_to_string(file_path);
    let Ok(text) = result
        else {
            println!("File ni gevonne...");
            return;
        };
    println!("{0}", text);
    let json_string = parse_and_print_nodes(text);
    println!("{}", json_string);
}
