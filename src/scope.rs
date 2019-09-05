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
        -> Result<&mut Self, Error>
        where S: Into<String>
    {
        let name = name.into();

        if args.len() == 0 {
            if self.functions.contains_key(&name) {
                return Err(Error::Define);
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

    pub fn get(&self, name: &str, stack: &Stack) -> Result<T, Error> {
        self.get_inner(name, stack)
            .or_else(|error|
                match self.parent {
                    Some(parent) => parent.get(name, stack),
                    None         => Err(error),
                }
            )
    }

    fn get_inner(&self, name: &str, stack: &Stack) -> Result<T, Error> {
        let mut node = self.functions.get(name)
            .ok_or_else(|| Error::Get)?;

        for expr in stack.peek() {
            let map = match node {
                Node::Type(map)   => map,
                Node::Function(f) => return Ok(f.clone()),
            };

            node = map.get(expr.get_type())
                .ok_or_else(|| Error::Get)?;
        }

        match node {
            Node::Type(_) => {
                Err(Error::Get)
            }
            Node::Function(f) => {
                Ok(f.clone())
            }
        }
    }
}


#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Define,
    Get,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Define =>
                write!(f, "Conflicting function definition found"),
            Error::Get => {
                // The call site wraps this error in its own error, as it has
                // more information about the error condition. This error should
                // never be formatted directly.
                unreachable!();
            }
        }
    }
}


pub enum Function<H> {
    Builtin(Builtin<H>),
    UserDefined(expr::List),
}

impl<H> Clone for Function<H> {
    fn clone(&self) -> Self {
        match self {
            Function::Builtin(f)     => Function::Builtin(f.clone()),
            Function::UserDefined(f) => Function::UserDefined(f.clone()),
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
        -> Result<(), Error>
    {
        let map = match self {
            Node::Type(map)   => map,
            Node::Function(_) => return Err(Error::Define),
        };

        let (&t, args) = match args.split_last() {
            Some(result) => result,

            None => {
                // We've run out of arguments to look at while unpacking the
                // already existing nodes on the path to our functions. This
                // means that a less specific function is already defined.
                return Err(Error::Define);
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
        types as t,
    };

    use super::{
        Error,
        Scope,
    };


    type Result = std::result::Result<(), Error>;


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
