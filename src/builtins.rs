use std::{
    collections::HashMap,
    ops::DerefMut,
};

use crate::{
    functions::Functions,
    parser::{
        Expression,
        List,
        Number,
    },
    stack::{
        self,
        Stack,
        Type,
    },
};


pub struct Builtins(HashMap<&'static str, Box<Builtin>>);

impl Builtins {
    pub fn new() -> Self {
        let mut b = HashMap::new();

        for builtin in builtins() {
            b.insert(builtin.name(), builtin);
        }

        Self(b)
    }

    pub fn get(&mut self, name: &str) -> Option<&mut (Builtin + 'static)> {
        self.0.get_mut(name).map(|builtin| builtin.deref_mut())
    }
}


pub trait Types {
    fn take(&mut self, _: &mut Stack) -> Result<(), stack::Error>;
}

impl<A> Types for (A,) where A: Type {
    fn take(&mut self, stack: &mut Stack) -> Result<(), stack::Error> {
        self.0 = stack.pop::<A>()?;

        Ok(())
    }
}

impl<A, B> Types for (A, B)
    where
        A: Type,
        B: Type,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), stack::Error> {
        self.1 = stack.pop::<B>()?;
        self.0 = stack.pop::<A>()?;

        Ok(())
    }
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn input(&mut self) -> &mut Types;
    fn run(&self, _: &mut Stack, _: &mut Functions);
}

macro_rules! impl_builtin {
    ($($ty:ident, $name:expr, $fn:ident, $input:ty;)*) => {
        fn builtins() -> Vec<Box<Builtin>> {
            vec![
                $($ty::new(),)*
            ]
        }

        $(
            pub struct $ty {
                input: $input,
            }

            impl $ty {
                fn new() -> Box<Builtin> {
                    Box::new($ty {
                        input: Default::default(),
                    })
                }
            }

            impl Builtin for $ty {
                fn name(&self) -> &'static str {
                    $name
                }

                fn input(&mut self) -> &mut Types {
                    &mut self.input
                }

                fn run(&self, stack: &mut Stack, functions: &mut Functions) {
                    $fn(&self.input, stack, functions)
                }
            }
        )*
    }
}

impl_builtin!(
    Print, "print",  print,  (Expression,);
    Define,"define", define, (List, List);

    Add, "+", add, (Number, Number);
    Mul, "*", mul, (Number, Number);
);

fn print((input,): &(Expression,), _: &mut Stack, _: &mut Functions) {
    match input {
        Expression::Number(number) => print!("{}", number),
        Expression::List(_)        => unimplemented!(),
        Expression::String(string) => print!("{}", string),
        Expression::Word(_)        => unimplemented!(),
    }
}

fn define(
    (body, name): &(List, List),
    _: &mut Stack,
    functions: &mut Functions,
) {
    assert_eq!(name.len(), 1);
    let name = name.clone().pop().unwrap();

    let name = match name {
        Expression::Word(word) => {
            word
        }
        expression => {
            panic!(
                "Unexpected expression: {:?}\n",
                expression,
            );
        }
    };

    functions.define(name, body.clone());
}

fn add((a, b): &(Number, Number), stack: &mut Stack, _: &mut Functions) {
    stack.push(Expression::Number(a + b));
}

fn mul((a, b): &(Number, Number), stack: &mut Stack, _: &mut Functions) {
    stack.push(Expression::Number(a * b));
}
