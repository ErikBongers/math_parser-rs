use std::cell::RefCell;
use std::rc::Rc;
use crate::errors::Error;
use crate::parser::{CodeBlock, Parser};
use crate::parser::nodes::print_nodes;
use crate::resolver::globals::Globals;
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::sources::Source;

mod tokenizer;
mod parser;
mod resolver;
mod errors;
mod functions;

pub fn parse_and_print_nodes (text: String) -> String {
    _parse_and_print_nodes(text, true)
}

pub fn parse(text: String) -> String {
    _parse_and_print_nodes(text, false)
}

//TODO
pub fn upload_source(text: String) -> i32 {
    12345
}

pub fn get_math_version() -> String {
    "0.0.0".to_string()
}

fn _parse_and_print_nodes (text: String, print: bool) -> String {
    let source = Source::new(text);
    let mut globals = Globals::new();
    globals.sources.push(source);//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
    let mut tok = PeekingTokenizer::new(globals.sources[0].text.as_str());
    let mut errors = Vec::<Error>::new();
    let scope = RefCell::new(Scope::new(&globals));
    let code_block = CodeBlock::new(scope);

    //parse
    let mut parser = Parser::new(&globals, &mut tok, &mut errors, code_block);
    parser.parse(false);
    let code_block: CodeBlock = parser.into();
    if(print) {
        for stmt in &code_block.statements {
            print_nodes(&stmt.node, 0);
        }
    }

    //resolve
    let mut resolver = Resolver {
        scope: code_block.scope.clone(),
        results: Vec::new(),
        errors: &mut errors, //TODO: make reference
        globals: &globals,
    };
    resolver.resolve(&code_block.statements);

    serde_json::to_string_pretty(&resolver).unwrap()
}

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::errors::{Error, ERROR_MAP, ErrorId};
    use crate::parser::{CodeBlock, Parser};
    use crate::resolver::globals::Globals;
    use crate::resolver::Resolver;
    use crate::resolver::scope::Scope;
    use crate::resolver::value::{Value, Variant};
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
    use crate::tokenizer::sources::Source;

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
    }

    #[test]
    fn test_assign_expr () {
        test_result("a=1;b=2;c=a+b", 3.0, "");
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
        test_result("(1m)mm", 1000.0, "mm");
        test_result("(1m).mm", 1000.0, "mm");
        test_result("(1.m)mm", 1000.0, "mm");
        test_result("1.m.mm", 1000.0, "mm");
        test_result("1m.mm", 1000.0, "mm");
    }

    #[test]
    fn test_nonsense () {
        //just test if it doesn't crash.
        get_results("");
        get_results(";");
        get_results("-");
    }

    fn test_result(text: &str, expected_result: f64, unit: &str) {
        let (results, _errors) = get_results(text);
        let value = results.last().expect("No result found.");
        let Variant::Numeric { number, .. } = &value.variant else {
            panic!("Result isn't a number.");
        };
        assert_eq!(number.to_double(), expected_result);
        assert_eq!(number.unit.id, unit);
    }

    fn test_error(text: &str, error_id: ErrorId) {
        let (_results, errors) = get_results(text);
        // let cnt = errors.len();
        // let cnt2 = errors.iter().count();
        // assert_ne!(cnt, 0);
        assert_ne!(errors.iter().filter(|&e| e.id == error_id).count(), 0);
    }

    fn get_results(text: &str) -> (Vec<Value>, Vec<Error>) {
        let source = Source::new(text.to_string());
        let mut tok = PeekingTokenizer::new(text);
        let mut globals = Globals::new();
        globals.sources.push(source);//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
        let mut scope = Scope::new(&globals);
        let mut code_block = CodeBlock::new(RefCell::new(scope));
        let mut errors: Vec<Error> = Vec::new();
        //parse
        let mut parser = Parser::new(&globals, &mut tok, &mut errors, code_block);
        parser.parse(false);
        let mut code_block: CodeBlock = parser.into();

        //resolve
        let mut resolver = Resolver {
            scope: code_block.scope.clone(),
            results: Vec::new(),
            errors: &mut errors,
            globals: &globals,
        };
        resolver.resolve(&code_block.statements);
        (resolver.results, errors)
    }
}