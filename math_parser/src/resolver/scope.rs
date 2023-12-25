use crate::resolver::globals::Globals;

pub struct Scope<'a> {
    pub globals: &'a Globals<'a>
}

impl<'a> Scope<'a> {
    pub fn new (globals: &'a mut Globals) -> Self {
        Scope {
            globals,
        }
    }
}