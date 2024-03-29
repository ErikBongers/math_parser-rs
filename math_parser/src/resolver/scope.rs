use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::date::DateFormat;
use crate::functions::{CustomFunctionDef, execute_custom_function, FunctionDef, FunctionView};
use crate::parser::nodes::{CodeBlock, FunctionDefExpr};
use crate::globals::Globals;
use crate::resolver::unit::{UnitsView};
use crate::resolver::value::Value;

#[derive(Clone, Copy)]
pub enum DecimalChar { Dot, Comma, Auto }

pub struct Scope {
    pub parent_scope: Option<Rc<RefCell<Scope>>>,
    pub var_defs: HashSet<String>,
    pub variables: HashMap<String, Value>,
    pub function_view: FunctionView,
    pub local_function_defs:  HashMap<String, CustomFunctionDef>,
    pub units_view: UnitsView,
    pub date_format: DateFormat,
    pub precision: f64,
    pub strict: bool,
    pub decimal_char: DecimalChar,
}

impl Scope {
    pub fn new (globals: &Globals) -> Self {
        Scope {
            parent_scope: None,
            var_defs: HashSet::new(),
            variables: HashMap::new(),
            function_view: globals.function_view.clone(),
            local_function_defs: HashMap::new(),
            units_view: globals.units_view.clone(),
            date_format: DateFormat::YMD,
            precision: 10.0_f64.powf(5.0),
            strict: false,
            decimal_char: DecimalChar::Auto,
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
            precision: scope.precision,
            strict: scope.strict,
            decimal_char: scope.decimal_char,

            //don't copy:
            local_function_defs: HashMap::new(),
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        })
    }

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

    #[inline]
    pub fn function_accessible(&self, id: &str) -> bool {
        self.function_view.ids.contains(id)
        //don't check parent: functions can be hidden within a block.
    }

    pub fn with_function<TReturnValue> (&self, id: &str, globals: &Globals, mut f: impl FnMut(&dyn FunctionDef) -> TReturnValue) -> Option<TReturnValue> {
        if let Some(fdef) = self.local_function_defs.get(id) {
           Some(f(fdef))
        } else {
            if let Some(parent_scope) = self.parent_scope.as_ref() {
                parent_scope.borrow().with_function(id, globals, f)
            } else {
                if let Some(fdef) = globals.global_function_defs.get(id) {
                    Some(f(fdef))
                } else {
                    None
                }
            }
        }
    }

    #[inline]
    pub fn function_exists(&self, function_name: &str, globals: &Globals) -> bool {
        self.with_function(function_name, globals, |_fd| ()).is_some()
    }
}