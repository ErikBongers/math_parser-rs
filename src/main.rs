use std::fs;
use math_parser::{get_math_version, parse_and_print_nodes};

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


#[cfg(test)]
mod test {
    use math_parser::test::{test_compiles, test_result, test_error, test_date};
    use math_parser::errors::ErrorId;
    #[test]
    fn test_numbers (){
        test_result("123.456", 123.456, "");
        test_result("0.123", 0.123, "");
        test_result("-1", -1.0, "");
    }

    #[test]
    fn test_dates (){
        test_date("#define dmy \n date(1,2,2022)", 1, 2, 2022);
        test_error("#define ymd \n date(1,2,2022)", ErrorId::InvDate);
        test_date("#define dmy \n '1-2-2022'", 1, 2, 2022);
        test_error("#define ymd \n '1-2-2022'", ErrorId::InvDateStr);
    }

    #[test]
    fn test_simple_expr (){
        test_result("1+2", 3.0, "");
        test_result("2*3", 6.0, "");
        test_result("6/3", 2.0, "");
        test_result("2^3", 8.0, "");
        test_result("|-8|", 8.0, "");
        test_result("10%12", 10.0, "");
        test_result("-10%12", -10.0, "");
        test_result("-10%%12", 2.0, "");

        test_result("0!", 1.0, "");
        test_result("1!", 1.0, "");
        test_result("2!", 2.0, "");
        test_result("5!", 120.0, "");
        test_error("5.3!", ErrorId::Expected);
        test_error("(-5)!", ErrorId::Expected);

        test_result("2*2*3", 12.0, "");
        test_result("7-2*3", 1.0, "");
        test_result("7-(2*3)", 1.0, "");
        test_result("(7-2)*3", 15.0, "");
    }

    #[test]
    fn test_implicit_mult () {
        test_result("a=2;2a", 4.0, "");
        test_result("a=2;(2)a", 4.0, "");
        test_result("a=2;20/2a", 5.0, "");
    }

    #[test]
    fn test_assign_expr () {
        test_result("a=1;b=2;c=a+b", 3.0, "");
        test_result("a=1;a+=2", 3.0, "");
        test_result("a=1mm;a.=", 1.0, "");
    }

    #[test]
    fn test_global_funcs () {
        test_result("abs(-1)", 1.0, "");
        test_result("a=1; a++", 2.0, "");
        test_result("a=2; a--", 1.0, "");
        test_result("sum(1,2,3)", 6.0, "");
        // test_result("sum(1,2, now())", 6.0, "");
        test_result("max(1,2,3)", 3.0, "");
        test_result("min(1,2,3)", 1.0, "");
        test_result("avg(1,2,3)", 2.0, "");
        test_result("first(1,2,3)", 1.0, "");
        test_result("last(1,2,3)", 3.0, "");
        test_result("first(reverse(1,2,3))", 3.0, "");
        test_result("first(sort(3,1, 2))", 1.0, "");
        test_compiles("now()");
    }

    #[test]
    fn test_function_calls () {
        test_result("abs(0-123)", 123.0, "");
    }

    #[test]
    fn test_number_formats () {
        test_result("123.dec;", 123.0, "");
        test_result("123.hex;", 123.0, "");
        test_result("123.oct;", 123.0, "");
        test_result("123.bin;", 123.0, "");
        test_result("0o173", 123.0, "");
        test_result("0O173", 123.0, "");
        test_result("0x7b", 123.0, "");
        test_result("0X7B", 123.0, "");
        test_result("0b1111011", 123.0, "");
        test_result("0B1111011", 123.0, "");
    }

    #[test]
    fn test_units () {
        test_result("(10.3+3).m-300cm", 10.3, "m");
        test_result("1L", 1.0, "L");
        test_result("1L+100ml", 1.1, "L");
        test_result("(1m)mm", 1000.0, "mm");
        test_result("(1m).mm", 1000.0, "mm");
        test_result("(1.m)mm", 1000.0, "mm");
        test_result("1.m.mm", 1000.0, "mm");
        test_result("1m.mm", 1000.0, "mm");
        test_result("sum(1mm, 2cm)", 0.021, "m");
        test_result("sum(1mm, 2cm).mm", 21.0, "mm");
        test_result("sum(1mm, 2cm)mm", 21.0, "mm");
        test_result("1mm.", 1.0, "");
    }

    #[test]
    fn test_nonsense () {
        //just test if it doesn't crash.
        test_compiles("");
        test_compiles(";");
        test_compiles("-");
        test_compiles("date(2022, 'sdf', 31)"); //fales in c++!
        test_compiles("a=1; { a=2; "); //block not closed.
    }

}