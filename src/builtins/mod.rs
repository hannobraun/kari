pub mod builtins;
pub mod context;


use std::collections::HashMap;

use builtins::Builtin;


pub struct Builtins(HashMap<&'static str, Builtin>);

impl Builtins {
    pub fn new() -> Self {
        let mut b = HashMap::new();

        for (name, builtin) in builtins::builtins() {
            b.insert(name, builtin);
        }

        Self(b)
    }

    pub fn builtin(&self, name: &str) -> Option<Builtin> {
        self.0
            .get(name)
            .map(|builtin| *builtin)
    }
}
