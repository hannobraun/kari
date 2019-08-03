use std::{
    collections::HashMap,
    ops::DerefMut,
    vec,
};

use crate::{
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
    fn place(&self, _: &mut Stack);
}

impl Types for () {
    fn take(&mut self, _: &mut Stack) -> Result<(), stack::Error> {
        Ok(())
    }

    fn place(&self, _: &mut Stack) {
        ()
    }
}

impl<A> Types for (A,)
    where
        A: Type + Clone,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), stack::Error> {
        self.0 = stack.pop::<A>()?;

        Ok(())
    }

    fn place(&self, stack: &mut Stack) {
        stack.push(self.0.clone());
    }
}

impl<A, B> Types for (A, B)
    where
        A: Type + Clone,
        B: Type + Clone,
{
    fn take(&mut self, stack: &mut Stack) -> Result<(), stack::Error> {
        self.1 = stack.pop::<B>()?;
        self.0 = stack.pop::<A>()?;

        Ok(())
    }

    fn place(&self, stack: &mut Stack) {
        stack.push(self.0.clone());
        stack.push(self.1.clone());
    }
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn input(&mut self) -> &mut Types;
    fn output(&self) -> &Types;
    fn defines(&mut self) -> vec::Drain<(String, List)>;
    fn run(&mut self);
}

macro_rules! impl_builtin {
    ($($ty:ident, $name:expr, $fn:ident, $input:ty => $output:ty;)*) => {
        fn builtins() -> Vec<Box<Builtin>> {
            vec![
                $($ty::new(),)*
            ]
        }

        $(
            pub struct $ty {
                input:   $input,
                output:  $output,
                defines: Vec<(String, List)>,
            }

            impl $ty {
                fn new() -> Box<Builtin> {
                    Box::new($ty {
                        input:   Default::default(),
                        output:  Default::default(),
                        defines: Vec::new(),
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

                fn output(&self) -> &Types {
                    &self.output
                }

                fn defines(&mut self) -> vec::Drain<(String, List)> {
                    self.defines.drain(..)
                }

                fn run(&mut self) {
                    $fn(&self.input, &mut self.output, &mut self.defines);
                }
            }
        )*
    }
}

impl_builtin!(
    Print, "print",  print,  (Expression,) => ();
    Define,"define", define, (List, List) => ();

    Add, "+", add, (Number, Number) => (Number,);
    Mul, "*", mul, (Number, Number) => (Number,);
);

fn print(
    (input,): &(Expression,),
    _       : &mut (),
    _       : &mut Vec<(String, List)>,
) {
    match input {
        Expression::Number(number) => print!("{}", number),
        Expression::List(_)        => unimplemented!(),
        Expression::String(string) => print!("{}", string),
        Expression::Word(_)        => unimplemented!(),
    }
}

fn define(
    (body, name): &(List, List),
    _           : &mut (),
    defines     : &mut Vec<(String, List)>,
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

    defines.push((name, body.clone()));
}

fn add(
    (a, b)   : &(Number, Number),
    (result,): &mut (Number,),
    _        : &mut Vec<(String, List)>,
) {
    *result = a + b;
}

fn mul(
    (a, b): &(Number, Number),
    (result,): &mut (Number,),
    _        : &mut Vec<(String, List)>,
) {
    *result = a * b;
}