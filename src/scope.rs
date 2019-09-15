use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    rc::Rc,
};

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        expr,
        span::Span,
        stack::Stack,
        types::{
            Type,
            Typed,
        },
    },
};


pub struct Scope<'r, T> {
    parent:    Option<&'r Scope<'r, T>>,
    functions: HashMap<String, Node<T>>,
}

impl<'r, T> Scope<'r, T>
    where T: Clone
{
    pub fn root() -> Self {
        Self {
            parent:    None,
            functions: HashMap::new(),
        }
    }

    pub fn child(&'r self) -> Self {
        Scope {
            parent:    Some(self),
            functions: HashMap::new(),
        }
    }

    pub fn define<S>(&mut self,
        name: S,
        args: &[&'static dyn Type],
        f:    T,
    )
        -> Result<&mut Self, DefineError>
        where S: Into<String>
    {
        let name = name.into();

        if args.len() == 0 {
            if self.functions.contains_key(&name) {
                return Err(DefineError);
            }

            self.functions.insert(
                name,
                Node::Function(f),
            );
            return Ok(self);
        }

        let node = self.functions
            .entry(name)
            .or_insert(Node::Type(HashMap::new()));

        node.insert(args, f)?;

        Ok(self)
    }

    pub fn get(&self, name: &str, stack: &Stack) -> Result<T, GetError> {
        self.get_inner(name, stack)
            .or_else(|error|
                match self.parent {
                    Some(parent) => parent.get(name, stack),
                    None         => Err(error),
                }
            )
    }

    fn get_inner(&self, name: &str, stack: &Stack) -> Result<T, GetError> {
        let mut node = self.functions.get(name)
            .ok_or_else(||
                GetError { candidates: self.candidates_for(name) }
            )?;

        for expr in stack.peek() {
            let map = match node {
                Node::Type(map)   => map,
                Node::Function(f) => return Ok(f.clone()),
            };

            node = map.get(expr.get_type())
                .ok_or_else(||
                    GetError { candidates: self.candidates_for(name) }
                )?;
        }

        match node {
            Node::Type(_) => {
                Err(GetError { candidates: self.candidates_for(name) })
            }
            Node::Function(f) => {
                Ok(f.clone())
            }
        }
    }

    fn candidates_for(&self, name: &str) -> Vec<Vec<&'static dyn Type>> {
        let mut candidates = Vec::new();

        if let Some(node) = self.functions.get(name) {
            node.all_paths(Vec::new(), &mut candidates);
        }

        candidates
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct DefineError;

impl fmt::Display for DefineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Conflicting function definition found")
    }
}


#[derive(Debug, Eq, PartialEq)]
pub struct GetError {
    pub candidates: Vec<Vec<&'static dyn Type>>,
}


pub enum Function<H> {
    Builtin(Builtin<H>),
    UserDefined {
        body: expr::List,
    }
}

impl<H> Clone for Function<H> {
    fn clone(&self) -> Self {
        match self {
            Function::Builtin(f) => {
                Function::Builtin(f.clone())
            }
            Function::UserDefined { body } => {
                Function::UserDefined {
                    body: body.clone(),
                }
            }
        }
    }
}


pub type Host<H> = Rc<RefCell<H>>;

pub type Builtin<H> =
    fn(Host<H>, &mut dyn Context<H>, &mut Scope<Function<H>>, Span)
        -> Result<(), context::Error>;


enum Node<T> {
    Type(HashMap<&'static dyn Type, Node<T>>),
    Function(T),
}

impl<T> Node<T> {
    fn insert(&mut self, args: &[&'static dyn Type], f: T)
        -> Result<(), DefineError>
    {
        let map = match self {
            Node::Type(map)   => map,
            Node::Function(_) => return Err(DefineError),
        };

        let (&t, args) = match args.split_last() {
            Some(result) => result,

            None => {
                // We've run out of arguments to look at while unpacking the
                // already existing nodes on the path to our functions. This
                // means that a less specific function is already defined.
                return Err(DefineError);
            }
        };

        if let Some(node) = map.get_mut(t) {
            return node.insert(args, f);
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
        paths:        &mut Vec<Vec<&'static dyn Type>>,
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


#[cfg(test)]
mod tests {
    use crate::data::{
        expr::{
            self,
            Expr as _,
        },
        span::Span,
        stack::Stack,
        types::{
            self as t,
            Type,
        },
    };

    use super::{
        DefineError,
        Scope,
    };


    type Result = std::result::Result<(), DefineError>;


    #[test]
    fn it_should_return_none_if_function_wasnt_defined() {
        let scope = Scope::<()>::root();
        let stack = Stack::new();

        let result = scope.get("a", &stack);

        assert!(result.is_err());
    }

    #[test]
    fn it_should_return_functions_that_were_defined() -> Result {
        let mut scope = Scope::root();
        let mut stack = Stack::new();

        scope
            .define("a", &[&t::Number, &t::Float], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let result = scope.get("a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_the_function_that_matches_the_types_on_the_stack()
        -> Result
    {
        let mut scope = Scope::root();
        let mut stack = Stack::new();

        scope
            .define("a", &[&t::Number, &t::Float ], 1)?
            .define("a", &[&t::Number, &t::Number], 2)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let result = scope.get("a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_function_without_args_regardless_of_stack() -> Result {
        let mut scope = Scope::root();
        let mut stack = Stack::new();

        scope
            .define("a", &[], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let result = scope.get("a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_return_list_of_candidates_if_function_doesnt_match_stack()
        -> Result
    {
        let mut scope = Scope::root();
        let mut stack = Stack::new();

        scope
            .define("a", &[&t::Number, &t::Float], 1)?
            .define("a", &[&t::Float, &t::Float],  2)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Number::new(0, Span::default()));

        let error = match scope.get("a", &stack) {
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
        let mut scope = Scope::root();

        let result = scope
            .define("a", &[&t::Number, &t::Number], 1)?
            .define("a", &[&t::Number, &t::Number], 2);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_more_specific_than_a_defined_function()
        -> Result
    {
        let mut scope = Scope::root();

        let result = scope
            .define("a", &[&t::Number, &t::Number], 1)?
            .define("a", &[&t::Number], 2);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn it_should_reject_no_arg_functions_if_name_is_already_taken() -> Result {
        // This is a special case of the previous test case. Functions with no
        // arguments are specially handled in the code, so we also need a
        // special test for them.

        let mut scope = Scope::root();

        let result = scope
            .define("a", &[&t::Number], 1)?
            .define("a", &[], 2);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_less_specific_than_a_defined_function()
        -> Result
    {
        let mut scope = Scope::root();

        let result = scope
            .define("a", &[&t::Number], 1)?
            .define("a", &[&t::Number, &t::Number], 2);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn it_should_return_functions_that_were_defined_in_parent_scope()
        -> Result
    {
        let mut scope = Scope::root();
        let mut stack = Stack::new();

        scope
            .define("a", &[&t::Number, &t::Float], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let result = scope.child().get("a", &stack);

        assert_eq!(result, Ok(1));
        Ok(())
    }

    #[test]
    fn it_should_not_return_functions_that_were_defined_in_child_scope()
        -> Result
    {
        let     scope = Scope::root();
        let mut stack = Stack::new();

        scope.child()
            .define("a", &[&t::Number, &t::Float], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let result = scope.get("a", &stack);

        assert!(result.is_err());
        Ok(())
    }
}
