use std::{
    collections::HashMap,
    fmt,
};

use crate::{
    stack::Stack,
    types::{
        Type,
        Typed,
    },
};


#[derive(Debug)]
pub struct Functions<T> {
    scopes:        HashMap<Scope, HashMap<String, Node<T>>>,
    root:          Scope,
    parents:       HashMap<Scope, Scope>,
    names:         HashMap<Scope, String>,
    next_scope_id: u64,
}

impl<T> Functions<T>
    where T: Clone
{
    pub fn new() -> Self {
        let root = Scope {
            id: 0,
        };

        let mut scopes = HashMap::new();
        scopes.insert(root, HashMap::new());

        let mut names = HashMap::new();
        names.insert(root, "<root>".into());

        Self {
            scopes,
            root,
            parents:       HashMap::new(),
            names,
            next_scope_id: 1,
        }
    }

    pub fn define<S>(&mut self,
        scope: Scope,
        name:  S,
        args:  &[&'static dyn Type],
        f:     T,
    )
        -> Result<&mut Self, DefineError>
        where S: Into<String>
    {
        let name = name.into();

        let functions = self.scopes.get_mut(&scope)
            .expect("Scope not found");

        if args.len() == 0 {
            if let Some(node) = functions.get(&name) {
                let mut conflicting = Vec::new();
                node.all_paths(Vec::new(), &mut conflicting);

                return Err(
                    DefineError {
                        name,
                        conflicting,
                        scope_id: scope.id,
                    }
                );
            }

            functions.insert(
                name,
                Node::Function(f),
            );
            return Ok(self);
        }

        let node = functions
            .entry(name.clone())
            .or_insert(Node::Type(HashMap::new()));

        node.insert(args, f)
            .map_err(|conflicting|
                DefineError {
                    name,
                    conflicting,
                    scope_id: scope.id,
                }
            )?;

        Ok(self)
    }

    pub fn get(&self, scope: Scope, name: &str, stack: &Stack)
        -> Result<T, GetError>
    {
        let mut scope = scope;

        loop {
            match self.get_inner(scope, name, stack) {
                Ok(function) => return Ok(function),

                Err(error) => {
                    match self.parents.get(&scope) {
                        Some(parent) => scope = *parent,
                        None         => return Err(error),
                    }
                }
            }
        }
    }

    fn get_inner(&self, scope: Scope, name: &str, stack: &Stack)
        -> Result<T, GetError>
    {
        let functions = self.scopes.get(&scope)
            .expect("Scope not found");

        let mut node = functions.get(name)
            .ok_or_else(||
                GetError {
                    candidates: self.candidates_for(&functions, name),
                    scope:      self.scope_name(scope),
                }
            )?;

        for expr in stack.peek() {
            let map = match node {
                Node::Type(map)   => map,
                Node::Function(f) => return Ok(f.clone()),
            };

            node = map.get(expr.get_type())
                .ok_or_else(||
                    GetError {
                        candidates: self.candidates_for(functions, name),
                        scope:      self.scope_name(scope),
                    }
                )?;
        }

        match node {
            Node::Type(_) => {
                Err(
                    GetError {
                        candidates: self.candidates_for(functions, name),
                        scope:      self.scope_name(scope),
                    }
                )
            }
            Node::Function(f) => {
                Ok(f.clone())
            }
        }
    }

    fn candidates_for(&self, functions: &HashMap<String, Node<T>>, name: &str)
        -> Signatures
    {
        let mut candidates = Vec::new();

        if let Some(node) = functions.get(name) {
            node.all_paths(Vec::new(), &mut candidates);
        }

        candidates
    }

    pub fn root_scope(&self) -> Scope {
        self.root
    }

    pub fn new_scope(&mut self, parent: Scope, name: impl Into<String>)
        -> Scope
    {
        assert!(self.next_scope_id < u64::max_value());

        let id = self.next_scope_id;
        self.next_scope_id += 1;

        let scope = Scope {
            id
        };
        self.scopes.insert(scope, HashMap::new());
        self.parents.insert(scope, parent);
        self.names.insert(scope, name.into());

        scope
    }

    fn scope_name(&self, scope: Scope) -> String {
        let mut scope = scope;

        let mut name = self.names.get(&scope)
            // Shouldn't panic. If the scope exists, the name must exist.
            .unwrap()
            .clone();

        while let Some(parent) = self.parents.get(&scope) {
            let parent_name = &self.names.get(&parent)
                // Shouldn't panic. If the scope exists, the name must exist.
                .unwrap();

            name.insert_str(0, " -> ");
            name.insert_str(0, parent_name);

            scope = *parent;
        }

        name
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Scope {
    id: u64,
}


#[derive(Debug)]
enum Node<T> {
    Type(HashMap<&'static dyn Type, Node<T>>),
    Function(T),
}

impl<T> Node<T> {
    fn insert(&mut self, args: &[&'static dyn Type], f: T)
        -> Result<(), Signatures>
    {
        let map = match self {
            Node::Type(map) => {
                map
            }
            Node::Function(_) => {
                return Err(
                    // We know there is one conflicting function, because we
                    // just loaded it from the map. We need to add an empty
                    // `Vec` for it to `conflicting`. Its type will be
                    // backfilled when the recursive `insert` calls return.
                    vec![Vec::new()],
                )
            }
        };

        let (&t, args) = match args.split_last() {
            Some(result) => result,

            None => {
                // We've run out of arguments to look at while unpacking the
                // already existing nodes on the path to our functions. This
                // means that a less specific function is already defined.

                let mut conflicting = Vec::new();
                self.all_paths(Vec::new(), &mut conflicting);

                return Err(conflicting);
            }
        };

        if let Some(node) = map.get_mut(t) {
            return node.insert(args, f)
                .map_err(|mut conflicting| {
                    for signature in &mut conflicting {
                        signature.insert(0, t);
                    }
                    conflicting
                });
        }

        let mut node = Node::Function(f);

        for &t in args {
            let mut map = HashMap::new();
            map.insert(
                t,
                node,
            );
            node = Node::Type(map);
        }

        map.insert(
            t,
            node,
        );

        Ok(())
    }

    fn all_paths(&self,
        current_path: Vec<&'static dyn Type>,
        paths:        &mut Signatures,
    ) {
        match self {
            Node::Type(map) => {
                for (ty, node) in map.iter() {
                    let mut path = current_path.clone();
                    path.insert(0, *ty);
                    node.all_paths(path, paths);
                }
            }
            Node::Function(_) => {
                paths.push(current_path);
            }
        }
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct DefineError {
    pub name:        String,
    pub conflicting: Signatures,
    pub scope_id:    u64,
}

impl fmt::Display for DefineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Conflicting function found defining `{}` in scope {}:\n",
            self.name, self.scope_id,
        )?;

        for conflicting in &self.conflicting {
            write!(f, "{:?}\n", conflicting)?;
        }

        Ok(())
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct GetError {
    pub candidates: Signatures,
    pub scope:      String,
}


pub type Signatures = Vec<Vec<&'static dyn Type>>;


#[cfg(test)]
mod tests {
    use crate::{
        stack::Stack,
        token::Span,
        types::{
            self as t,
            Type,
        },
        value::{
            self,
            Value as _,
        },
    };

    use super::{
        DefineError,
        Functions,
    };


    type Result = std::result::Result<(), DefineError>;


    #[test]
    fn it_should_return_none_if_function_wasnt_defined() {
        let functions = Functions::<()>::new();
        let scope     = functions.root_scope();
        let stack     = Stack::new();

        let result = functions.get(scope, "a", &stack);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_return_functions_that_were_defined() -> Result {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();
        let mut stack     = Stack::new();

        functions
            .define(scope, "a", &[&t::Number, &t::Float], 1)?;
        stack
            .push(value::Number::new(0, Span::default()))
            .push(value::Float::new(0.0, Span::default()));

        let result = functions.get(scope, "a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_the_function_that_matches_the_types_on_the_stack()
        -> Result
    {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();
        let mut stack     = Stack::new();

        functions
            .define(scope, "a", &[&t::Number, &t::Float ], 1)?
            .define(scope, "a", &[&t::Number, &t::Number], 2)?;
        stack
            .push(value::Number::new(0, Span::default()))
            .push(value::Float::new(0.0, Span::default()));

        let result = functions.get(scope, "a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_function_without_args_regardless_of_stack() -> Result {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();
        let mut stack     = Stack::new();

        functions
            .define(scope, "a", &[], 1)?;
        stack
            .push(value::Number::new(0, Span::default()))
            .push(value::Float::new(0.0, Span::default()));

        let result = functions.get(scope, "a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_list_of_candidates_if_function_doesnt_match_stack()
        -> Result
    {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();
        let mut stack     = Stack::new();

        functions
            .define(scope, "a", &[&t::Number, &t::Float], 1)?
            .define(scope, "a", &[&t::Float, &t::Float],  2)?;
        stack
            .push(value::Number::new(0, Span::default()))
            .push(value::Number::new(0, Span::default()));

        let error = match functions.get(scope, "a", &stack) {
            Ok(_)      => panic!("Expected error"),
            Err(error) => error,
        };

        assert!(
            error.candidates.contains(&vec![&t::Number as &dyn Type, &t::Float])
        );
        assert!(
            error.candidates.contains(&vec![&t::Float, &t::Float])
        );

        Ok(())
    }

    #[test]
    fn it_should_reject_functions_that_are_already_defined() -> Result {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();

        let result = functions
            .define(scope, "a", &[&t::Number, &t::Number], 1)?
            .define(scope, "a", &[&t::Number, &t::Number], 2);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_more_specific_than_a_defined_function()
        -> Result
    {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();

        let err = functions
            .define(scope, "a", &[&t::Number, &t::Number], 1)?
            .define(scope, "a", &[&t::Number], 2)
            .unwrap_err();

        assert_eq!(err.name, String::from("a"));
        assert_eq!(err.conflicting.len(), 1);
        assert!(err.conflicting.contains(&vec![&t::Number, &t::Number]));

        Ok(())
    }

    #[test]
    fn it_should_reject_no_arg_functions_if_name_is_already_taken() -> Result {
        // This is a special case of the previous test case. Functions with no
        // arguments are specially handled in the code, so we also need a
        // special test for them.

        let mut functions = Functions::new();
        let     scope     = functions.root_scope();

        let err = functions
            .define(scope, "a", &[&t::Number], 1)?
            .define(scope, "a", &[], 2)
            .unwrap_err();

        assert_eq!(err.name, String::from("a"));
        assert_eq!(err.conflicting.len(), 1);
        assert!(err.conflicting.contains(&vec![&t::Number]));

        Ok(())
    }

    #[test]
    fn it_should_reject_functions_less_specific_than_a_defined_function()
        -> Result
    {
        let mut functions = Functions::new();
        let     scope     = functions.root_scope();

        let err = functions
            .define(scope, "a", &[&t::Number], 1)?
            .define(scope, "a", &[&t::Number, &t::Number], 2)
            .unwrap_err();

        assert_eq!(err.name, String::from("a"));
        assert_eq!(err.conflicting.len(), 1);
        assert!(err.conflicting.contains(&vec![&t::Number]));

        Ok(())
    }

    #[test]
    fn it_should_find_function_defined_in_parent_scope()
        -> Result
    {
        let mut functions = Functions::new();
        let     stack     = Stack::new();

        let parent_scope = functions.root_scope();
        let child_scope  = functions.new_scope(parent_scope, "child");

        functions
            .define(parent_scope, "a", &[], 1)?;

        let result = functions.get(child_scope, "a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_not_find_function_defined_in_child_scope()
        -> Result
    {
        let mut functions = Functions::new();
        let     stack     = Stack::new();

        let parent_scope = functions.root_scope();
        let child_scope  = functions.new_scope(parent_scope, "child");

        functions
            .define(child_scope, "a", &[], 1)?;

        let result = functions.get(parent_scope, "a", &stack);

        assert!(result.is_err());
        Ok(())
    }
}
