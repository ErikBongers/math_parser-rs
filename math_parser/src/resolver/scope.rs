use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::functions::{CustomFunctionDef, execute_custom_function, FunctionView};
use crate::parser::date::date::DateFormat;
use crate::parser::nodes::{CodeBlock, FunctionDefExpr};
use crate::resolver::globals::Globals;
use crate::resolver::unit::UnitsView;
use crate::resolver::value::Value;

pub struct Scope {
    pub parent_scope: Option<Rc<RefCell<Scope>>>,
    pub var_defs: HashSet<String>,
    pub variables: HashMap<String, Value>,
    pub function_view: FunctionView,
    pub local_function_defs:  HashMap<String, CustomFunctionDef>,
    pub units_view: UnitsView,
    pub date_format: DateFormat,
}

impl Scope {
    pub fn new (globals: &Globals) -> Self {
        Scope {
            parent_scope: None,
            var_defs: HashSet::new(),
            variables: HashMap::new(),
            function_view: FunctionView { ids: HashSet::new()},
            local_function_defs: HashMap::new(),
            units_view: globals.units_view.as_ref().unwrap().clone(),
            date_format: DateFormat::YMD,
        }
    }

    pub fn copy_for_block(scope: &Rc<RefCell<Scope>>) -> RefCell<Scope> {
        let rc_scope = scope.clone();
        let scope = scope.borrow();
        RefCell::new(Scope {
            parent_scope: Some(rc_scope),
            function_view: scope.function_view.clone(),
            units_view: scope.units_view.clone(),
            date_format: scope.date_format,

            //don't copy variables.
            local_function_defs: HashMap::new(),
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        })
    }

    // pub fn get_local_function(&self, id: &str) -> Option<&CustomFunctionDef> {
    //     //TODO: filter the found function based on function_view
    //     if self.local_function_defs.contains_key(id) {
    //         self.local_function_defs.get(id)
    //     } else {
    //         if let Some(parent) = &self.parent_scope {
    //             parent.borrow().get_local_function(id)
    //         } else {
    //             None
    //         }
    //     }
    // }

    pub fn add_local_function(&mut self, code_block: CodeBlock, function_def_expr: &FunctionDefExpr) {
        let func = CustomFunctionDef {
            code_block,
            function_def_expr: function_def_expr.clone(),
            name: function_def_expr.id.clone(),
            min_args: function_def_expr.arg_names.len(),
            max_args: function_def_expr.arg_names.len(),
            execute: execute_custom_function
        };
        self.local_function_defs.insert(func.name.clone(), func);
    }

    pub fn var_exists(&self, id: &str, globals: &Globals) -> bool {
        if self.variables.contains_key(id) {
            return true;
        }
        globals.constants.contains_key(id)
    }

    pub fn get_var<'a>(&'a self, id: &str, globals: &'a Globals) -> &'a Value {
        if self.variables.contains_key(id) {
            return &self.variables[id];
        }
        &globals.constants[id]
    }
}