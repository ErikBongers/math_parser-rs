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
    let scope = RefCell::new(Scope::new());
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
    use crate::errors::Error;
    use crate::parser::{CodeBlock, Parser};
    use crate::resolver::globals::Globals;
    use crate::resolver::Resolver;
    use crate::resolver::scope::Scope;
    use crate::resolver::value::{Value, Variant};
    use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
    use crate::tokenizer::sources::Source;

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

    #[test]
    fn test_nonsense () {
        //just test if it doesn't crash.
        get_results("", 0.0, "");
        get_results(";", 0.0, "");
        get_results("-", 0.0, "");
    }

    fn test_result(text: &str, expected_result: f64, unit: &str) {
        let results = get_results(text, expected_result, unit);
        let value = results.last().expect("No result found.");
        let Variant::Number { number, .. } = &value.variant else {
            panic!("Result isn't a number.");
        };
        assert_eq!(number.to_double(), expected_result);
        assert_eq!(number.unit.id, unit);
    }

    fn get_results(text: &str, expected_result: f64, unit: &str) -> Vec<Value> {
        let source = Source::new(text.to_string());
        let mut tok = PeekingTokenizer::new(text);
        let mut globals = Globals::new();
        globals.sources.push(source);//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
        let mut scope = Scope::new();
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
        resolver.results
    }
}