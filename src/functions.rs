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
        span::Span,
        stack::Stack,
        types::{
            Type,
            Typed,
        },
    },
};


#[derive(Debug)]
pub struct Functions<T> {
    functions: HashMap<String, Node<T>>,
}

impl<T> Functions<T>
    where T: Clone
{
    pub fn new() -> Self {
        Self {
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
                return Err(Error::FunctionAlreadyDefined);
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

    pub fn get(&self, name: &str, stack: &Stack) -> Option<T> {
        let mut node = self.functions.get(name)?;

        for expr in stack.peek() {
            let map = match node {
                Node::Type(map)   => map,
                Node::Function(f) => return Some(f.clone()),
            };

            node = map.get(expr.get_type())?;
        }

        match node {
            Node::Type(_)     => None,
            Node::Function(f) => Some(f.clone()),
        }
    }
}


#[derive(Debug)]
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
            Node::Function(_) => return Err(Error::FunctionAlreadyDefined),
        };

        let (&t, args) = match args.split_last() {
            Some(result) => result,

            None => {
                // We've run out of arguments to look at while unpacking the
                // already existing nodes on the path to our functions. This
                // means that a less specific function is already defined.
                return Err(Error::FunctionAlreadyDefined);
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


#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    FunctionAlreadyDefined,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FunctionAlreadyDefined =>
                write!(f, "Function already defined"),
        }
    }
}


pub type Builtin =
    fn(&mut dyn Context, Span) -> Result<(), context::Error>;
pub type Extension<Host> =
    fn(Rc<RefCell<Host>>, &mut dyn Context, Span) -> Result<(), context::Error>;


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
        Functions,
    };


    #[test]
    fn it_should_return_none_if_function_wasnt_defined() {
        let functions = Functions::<()>::new();
        let stack     = Stack::new();

        let function = functions.get("a", &stack);

        assert_eq!(function, None);
    }

    #[test]
    fn it_should_return_functions_that_were_defined() -> Result<(), Error> {
        let mut functions = Functions::new();
        let mut stack     = Stack::new();

        functions
            .define("a", &[&t::Number, &t::Float], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let function = functions.get("a", &stack);

        assert_eq!(function, Some(1));
        Ok(())
    }

    #[test]
    fn it_should_return_the_function_that_matches_the_types_on_the_stack()
        -> Result<(), Error>
    {
        let mut functions = Functions::new();
        let mut stack     = Stack::new();

        functions
            .define("a", &[&t::Number, &t::Float ], 1)?
            .define("a", &[&t::Number, &t::Number], 2)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let function = functions.get("a", &stack);

        assert_eq!(function, Some(1));
        Ok(())
    }

    #[test]
    fn it_should_return_function_without_args_regardless_of_stack()
        -> Result<(), Error>
    {
        let mut functions = Functions::new();
        let mut stack     = Stack::new();

        functions
            .define("a", &[], 1)?;
        stack
            .push(expr::Number::new(0, Span::default()))
            .push(expr::Float::new(0.0, Span::default()));

        let function = functions.get("a", &stack);

        assert_eq!(function, Some(1));
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_that_are_already_defined()
        -> Result<(), Error>
    {
        let mut functions = Functions::new();

        let result = functions
            .define("a", &[&t::Number, &t::Number], 1)?
            .define("a", &[&t::Number, &t::Number], 2);

        assert_eq!(result.map(|_| ()), Err(Error::FunctionAlreadyDefined));
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_more_specific_than_a_defined_function()
        -> Result<(), Error>
    {
        let mut functions = Functions::new();

        let result = functions
            .define("a", &[&t::Number, &t::Number], 1)?
            .define("a", &[&t::Number], 2);

        assert_eq!(result.map(|_| ()), Err(Error::FunctionAlreadyDefined));
        Ok(())
    }

    #[test]
    fn it_should_reject_no_arg_functions_if_name_is_already_taken()
        -> Result<(), Error>
    {
        // This is a special case of the previous test case. Functions with no
        // arguments are specially handled in the code, so we also need a
        // special test for them.

        let mut functions = Functions::new();

        let result = functions
            .define("a", &[&t::Number], 1)?
            .define("a", &[], 2);

        assert_eq!(result.map(|_| ()), Err(Error::FunctionAlreadyDefined));
        Ok(())
    }

    #[test]
    fn it_should_reject_functions_less_specific_than_a_defined_function()
        -> Result<(), Error>
    {
        let mut functions = Functions::new();

        let result = functions
            .define("a", &[&t::Number], 1)?
            .define("a", &[&t::Number, &t::Number], 2);

        assert_eq!(result.map(|_| ()), Err(Error::FunctionAlreadyDefined));
        Ok(())
    }
}
