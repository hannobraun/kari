use std::collections::HashMap;

use crate::{
    parser::Expression,
    stack::{
        self,
        Number,
        Quote,
        Stack,
        Value,
    },
};


pub struct Functions(HashMap<String, Function>);

impl Functions {
    pub fn new() -> Self {
        let mut functions = HashMap::new();

        macro_rules! insert {
            ($name:expr, $builtin:expr) => {
                functions.insert(
                    String::from($name),
                    Function::Builtin(&$builtin)
                );
            }
        }

        insert!("print",  print);
        insert!("define", define);

        insert!("+", add);
        insert!("*", mul);

        Self(functions)
    }

    pub fn define(&mut self, name: String, body: Quote) {
        self.0.insert(name, Function::Quote(body));
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
    Quote(Quote),
}


pub type Builtin = Fn(&mut Stack, &mut Functions) -> Result<(), stack::Error>;


pub fn print(stack: &mut Stack, _: &mut Functions) -> Result<(), stack::Error> {
    let arg = stack.pop::<Value>()?;
    print!("{}", arg);

    Ok(())
}

pub fn define(stack: &mut Stack, functions: &mut Functions)
    -> Result<(), stack::Error>
{
    let mut name = stack.pop::<Quote>()?;
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

    let body = stack.pop::<Quote>()?;

    functions.define(name, body);

    Ok(())
}

pub fn add(stack: &mut Stack, _: &mut Functions) -> Result<(), stack::Error> {
    let b = stack.pop::<Number>()?;
    let a = stack.pop::<Number>()?;

    stack.push(Value::Number(a + b));

    Ok(())
}

pub fn mul(stack: &mut Stack, _: &mut Functions) -> Result<(), stack::Error> {
    let b = stack.pop::<Number>()?;
    let a = stack.pop::<Number>()?;

    stack.push(Value::Number(a * b));

    Ok(())
}
