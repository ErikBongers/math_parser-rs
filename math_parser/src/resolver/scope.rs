use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::functions::{CustomFunctionDef, FunctionView};
use crate::resolver::globals::Globals;
use crate::resolver::value::Value;

pub struct Scope<'a> {
    pub globals: &'a Globals<'a>,
    pub parent_scope: Option<Rc<Scope<'a>>>,
    pub var_defs: HashSet<String>,
    pub variables: HashMap<String, Value>,
    pub function_view: FunctionView,
    pub local_function_defs:  HashMap<&'a str, CustomFunctionDef<'a>>,
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

    pub fn copy_for_block(scope: &Rc<Scope<'a>>) -> Rc<Scope<'a>> {
        Rc::new(Scope {
            globals: scope.globals,
            parent_scope: Some(scope.clone()),
            function_view: scope.function_view.clone(),

            //don't copy variables.
            local_function_defs: HashMap::new(),
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        })
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
}