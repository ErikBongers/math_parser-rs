use crate::resolver::value::{Value, Variant};

pub fn recursive_iter(value_list: &Vec<Value>) -> RecursiveIter {
    RecursiveIter {
        inner: VecValueIterHolder { iter: Box::new(value_list.iter()), child_iter: None },
    }
}

pub struct RecursiveIter<'a> {
    inner: VecValueIterHolder<'a>,
}

impl<'a> Iterator for RecursiveIter<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<&'a Value> {
        self.inner.next()
    }
}

struct VecValueIterHolder<'a> {
    iter: Box<dyn Iterator<Item=&'a Value> + 'a>,
    child_iter: Option<Box<RecursiveIter<'a>>>,
}

impl<'a> Iterator for VecValueIterHolder<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<&'a Value> {
        //first check if we're iterating childs.
        if let Some(ref mut child_iter) = self.child_iter {
            let the_next = child_iter.next();
            if the_next.is_some() {
                return the_next;
            }
            //child_iter is finished. Remove it.
            self.child_iter = None;
            return self.local_next();
        } else {
            return self.local_next();
        }
    }
}

impl<'a> VecValueIterHolder<'a> {
    pub fn local_next(&mut self) -> Option<&'a Value> {
        let Some(value) = self.iter.next() else {
            return None;
        };
        match &value.variant {
            Variant::List { values } => {
                if values.is_empty() {
                    return self.next();
                }
                self.child_iter = Some(Box::new(RecursiveIter { inner: VecValueIterHolder { iter: Box::new(values.iter()), child_iter: None } }));
                self.child_iter.as_mut().unwrap().next() //unwrap: we just set the child_iter.
            },
            _ => Some(value)
        }
    }
}

