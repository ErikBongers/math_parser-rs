use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::resolver::globals::Globals;
use crate::resolver::value::Value;

pub struct Scope<'a> {
    pub globals: &'a Globals<'a>,
    pub parent_scope: Option<Rc<Scope<'a>>>,
    pub var_defs: HashSet<String>,
    pub variables: HashMap<String, Value>,
}

impl<'a> Scope<'a> {
    pub fn new (globals: &'a Globals) -> Self {
        Scope {
            globals,
            parent_scope: None,
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        }
    }

    pub fn copy_for_block(scope: &Rc<Scope<'a>>) -> Rc<Scope<'a>> {
        Rc::new(Scope {
            globals: scope.globals,
            parent_scope: Some(scope.clone()),
            //don't copy variables.
            var_defs: HashSet::new(),
            variables: HashMap::new(),
        })
    }
}