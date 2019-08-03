use std::collections::HashMap;

use crate::{
    parser::{
        Expression,
        List,
        Number,
    },
    stack::{
        self,
        Stack,
    },
};


pub struct Functions(HashMap<String, Function>);

impl Functions {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        let builtins = [
            &Print as &Builtin,
            &Define,

            &Add,
            &Mul,
        ];

        for &builtin in &builtins {
            functions.insert(
                String::from(builtin.name()),
                Function::Builtin(builtin)
            );
        }

        Self(functions)
    }

    pub fn define(&mut self, name: String, body: List) {
        self.0.insert(name, Function::List(body));
    }

    pub fn get(&self, name: &str) -> Option<&Function> {
        self.0.get(name)
    }
}


pub enum Function {
    Builtin(&'static Builtin),
    List(List),
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn run(&self, _: &mut Stack, _: &mut Functions) -> Result<(), stack::Error>;
}

macro_rules! impl_builtin {
    ($($ty:ident, $name:expr, $fn:ident;)*) => {
        $(
            pub struct $ty;

            impl Builtin for $ty {
                fn name(&self) -> &'static str {
                    $name
                }

                fn run(&self, stack: &mut Stack, functions: &mut Functions)
                    -> Result<(), stack::Error>
                {
                    $fn(stack, functions)
                }
            }
        )*
    }
}

impl_builtin!(
    Print, "print",  print;
    Define,"define", define;

    Add, "+", add;
    Mul, "*", mul;
);

fn print(stack: &mut Stack, _: &mut Functions)
    -> Result<(), stack::Error>
{
    match stack.pop::<Expression>()? {
        Expression::Number(number) => print!("{}", number),
        Expression::List(_)        => unimplemented!(),
        Expression::String(string) => print!("{}", string),
        Expression::Word(_)        => unimplemented!(),
    }

    Ok(())
}

fn define(stack: &mut Stack, functions: &mut Functions)
    -> Result<(), stack::Error>
{
    let mut name = stack.pop::<List>()?;
    assert_eq!(name.len(), 1);
    let name = name.pop().unwrap();

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

    let body = stack.pop::<List>()?;

    functions.define(name, body);

    Ok(())
}

fn add(stack: &mut Stack, _: &mut Functions)
    -> Result<(), stack::Error>
{
    let b = stack.pop::<Number>()?;
    let a = stack.pop::<Number>()?;

    stack.push(Expression::Number(a + b));

    Ok(())
}

fn mul(stack: &mut Stack, _: &mut Functions)
    -> Result<(), stack::Error>
{
    let b = stack.pop::<Number>()?;
    let a = stack.pop::<Number>()?;

    stack.push(Expression::Number(a * b));

    Ok(())
}
