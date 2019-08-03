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

    pub fn get(&self, name: &str) -> Option<Function> {
        self.0
            .get(name)
            .cloned()
    }
}


#[derive(Clone)]
pub enum Function {
    Builtin(&'static Builtin),
    List(List),
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn run(&self, _: &mut Stack, _: &mut Functions) -> Result<(), stack::Error>;
}


pub struct Print;

impl Builtin for Print {
    fn name(&self) -> &'static str {
        "print"
    }

    fn run(&self, stack: &mut Stack, functions: &mut Functions)
        -> Result<(), stack::Error>
    {
        print(stack, functions)
    }
}


pub struct Define;

impl Builtin for Define {
    fn name(&self) -> &'static str {
        "define"
    }

    fn run(&self, stack: &mut Stack, functions: &mut Functions)
        -> Result<(), stack::Error>
    {
        define(stack, functions)
    }
}


pub struct Add;

impl Builtin for Add {
    fn name(&self) -> &'static str {
        "+"
    }

    fn run(&self, stack: &mut Stack, functions: &mut Functions)
        -> Result<(), stack::Error>
    {
        add(stack, functions)
    }
}


pub struct Mul;

impl Builtin for Mul {
    fn name(&self) -> &'static str {
        "*"
    }

    fn run(&self, stack: &mut Stack, functions: &mut Functions)
        -> Result<(), stack::Error>
    {
        mul(stack, functions)
    }
}


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
