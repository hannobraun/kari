use std::collections::HashMap;


pub struct Functions<T: Copy>(HashMap<String, T>);

impl<T> Functions<T> where T: Copy {
    pub fn new(functions: HashMap<String, T>) -> Self {
        Self(functions)
    }

    pub fn get(&self, name: &str) -> Option<T> {
        self.0
            .get(name)
            .map(|function| *function)
    }
}
