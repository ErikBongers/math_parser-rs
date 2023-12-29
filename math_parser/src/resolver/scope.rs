use std::collections::{HashMap, HashSet};
use crate::functions::{CustomFunctionDef, execute_custom_function, FunctionView};
use crate::parser::CodeBlock;
use crate::parser::nodes::FunctionDefExpr;
use crate::resolver::globals::Globals;
use crate::resolver::value::Value;

pub struct Scope<'a> {
    pub globals: &'a Globals<'a>,
    pub parent_scope: Option<&'a Scope<'a>>,
    pub var_defs: HashSet<String>,
    pub variables: HashMap<String, Value>,
    pub function_view: FunctionView,
    pub local_function_defs:  HashMap<String, CustomFunctionDef<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new (globals: &'a Globals) -> Self {
        Scope {
            globals,
            parent_scope: None,
            var_defs: HashSet::new(),
            variables: HashMap::new(),
            function_view: FunctionView { ids: HashSet::new()},
            local_function_defs: HashMap::new(),
        }
    }

    pub fn copy_for_block(scope: &'a Scope<'a>) -> Scope<'a> {
        Scope {
            globals: scope.globals,
            parent_scope: Some(scope),
            function_view: scope.function_view.clone(),

            //don't copy variables.
            local_function_defs: HashMap::new(),
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        }
    }

    pub fn get_local_function(&self, id: &str) -> Option<&CustomFunctionDef> {
        //TODO: filter the found function based on function_view
        if self.local_function_defs.contains_key(id) {
            self.local_function_defs.get(id)
        } else {
            if let Some(parent) = &self.parent_scope {
                parent.get_local_function(id)
            } else {
                None
            }
        }
    }

    pub fn add_local_function(&mut self, code_block: CodeBlock<'a>, function_def_expr: &'a FunctionDefExpr) {
        // let func = CustomFunctionDef {
        //     code_block,
        //     function_def_expr,
        //     name: function_def_expr.id.clone(),
        //     min_args: function_def_expr.arg_names.len(),
        //     max_args: function_def_expr.arg_names.len(),
        //     execute: execute_custom_function,
        // };
        // self.local_function_defs.insert(func.name.clone(), func);
    }
}