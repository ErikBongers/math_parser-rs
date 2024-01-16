use std::cell::RefCell;
use crate::errors::Error;
use crate::parser::{Parser};
use crate::parser::nodes::{CodeBlock, print_nodes};
use crate::resolver::globals::Globals;
use crate::resolver::Resolver;
use crate::resolver::scope::Scope;
use crate::tokenizer::peeking_tokenizer::PeekingTokenizer;
use crate::tokenizer::sources::Source;

mod tokenizer;
mod parser;
mod resolver;
pub mod errors; //ErrorId is public
mod functions;


pub struct Api {
    globals: Globals,
}

impl Api {
    pub fn new() -> Api {
        Api {
            globals: Globals::new(),
        }
    }

    pub fn set_source(&mut self, name: String, text: String) -> i32 {
        self.globals.set_source(name, text)
    }

    pub fn parse(&mut self, start_script_id: String, main_script_id: String) -> String {
        //global stuff
        let mut errors = Vec::<Error>::new();
        let scope = RefCell::new(Scope::new(&self.globals));
        let mut code_block = CodeBlock::new(scope);

        //parse start script first?
        if(start_script_id != "") {
            let Some(source) = self.globals.get_source_by_name(&start_script_id) else {
                panic!("TODO: return source file {} not found as an error -> json", start_script_id);
            };
            let mut tok = PeekingTokenizer::new(source);

            //parse
            let mut parser = Parser::new(&self.globals, &mut tok, &mut errors, code_block);
            parser.parse(false);
            code_block = parser.into();
        }

        let Some(source) = self.globals.get_source_by_name(&main_script_id) else {
            panic!("TODO: return source file {} not found as an error -> json", main_script_id);
        };
        let mut tok = PeekingTokenizer::new(source);

        //parse
        let mut parser = Parser::new(&self.globals, &mut tok, &mut errors, code_block);
        parser.parse(false);
        code_block = parser.into();

        //resolve
        let mut resolver = Resolver {
            scope: code_block.scope.clone(),
            results: Vec::new(),
            errors: &mut errors,
            globals: &self.globals,
        };
        resolver.resolve(&code_block.statements);

        serde_json::to_string_pretty(&resolver).unwrap()
    }

    pub fn get_math_version() -> String {
        let major = env!("MATH_MAJOR");
        let minor = env!("MATH_MINOR");
        let build = env!("MATH_BUILD");
        format!("{major}.{minor}.{build}")
    }
}

pub fn parse_2_files(text1: String, text2: String) -> String {
    let mut api = Api::new();

    api.set_source("source1".to_string(), text1);
    api.set_source("source2".to_string(), text2);

    api.parse("source1".to_string(), "source2".to_string())
}

pub fn parse_1_file(text1: String) -> String {
    let mut api = Api::new();

    api.set_source("source1".to_string(), text1);

    api.parse("".to_string(), "source1".to_string())
}

/// Public api with test functions to use in external tests.
/// Having the tests external speeds up rebuilding as the tests are not part of the math_parser lib crate.
pub mod test {
    use std::cell::RefCell;
    use crate::{
        errors::{ Error, ErrorId },
        parser::{Parser},
        resolver::{ globals::Globals, Resolver, scope::Scope, value::{Value, Variant} },
        tokenizer::{ peeking_tokenizer::PeekingTokenizer, sources::Source }
    };
    use crate::parser::nodes::CodeBlock;

    pub fn test_result(text: &str, expected_result: f64, unit: &str) {
        let (results, _errors) = get_results(text);
        let value = results.last().expect("No result found.");
        let Variant::Numeric { number, .. } = &value.variant else {
            panic!("Result isn't a number.");
        };
        //round decimals:
        let val = number.to_double();
        let precision = 10000000.0;
        let val = (val*precision).round()/precision;
        assert_eq!(val, expected_result);
        assert_eq!(number.unit.id, unit);
    }

    pub fn test_date(text: &str, day: i8, month: i32, year: i32) {
        let (results, _errors) = get_results(text);
        let value = results.last().expect("No result found.");
        let Variant::Date { date, .. } = &value.variant else {
            panic!("Result isn't a date.");
        };
        assert_eq!(date.day, day);
        assert_eq!(date.month as i32, month);
        assert_eq!(date.year, year);
    }

    pub fn test_error(text: &str, error_id: ErrorId) {
        let (_results, errors) = get_results(text);
        // let cnt = errors.len();
        // let cnt2 = errors.iter().count();
        // assert_ne!(cnt, 0);
        assert_ne!(errors.iter().filter(|&e| e.id == error_id).count(), 0);
    }

    pub fn get_results(text: &str) -> (Vec<Value>, Vec<Error>) {
        let mut globals = Globals::new();
        let src_name = "src1";
        globals.set_source(src_name.to_string(), text.to_string());//TODO: this could be forgotten: allow only parsing and resolving of registered sources.
        let mut tok = PeekingTokenizer::new(globals.get_source_by_name(src_name).unwrap()); //unwrap ok: we just pushed a source.
        let scope = Scope::new(&globals);
        let code_block = CodeBlock::new(RefCell::new(scope));
        let mut errors: Vec<Error> = Vec::new();
        //parse
        let mut parser = Parser::new(&globals, &mut tok, &mut errors, code_block);
        parser.parse(false);
        let code_block: CodeBlock = parser.into();

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

    pub fn test_compiles(text: &str) {
        get_results(text);
    }
}