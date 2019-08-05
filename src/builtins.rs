use std::{
    collections::HashMap,
    vec,
};

use crate::{
    context::{
        self,
        Context,
    },
    parser::{
        Expression,
        List,
        Number,
    },
};


pub struct Builtins(HashMap<&'static str, &'static Builtin>);

impl Builtins {
    pub fn new() -> Self {
        let mut b = HashMap::new();

        for builtin in builtins() {
            b.insert(builtin.name(), builtin);
        }

        Self(b)
    }

    pub fn builtin(&self, name: &str) -> Option<&'static (Builtin + 'static)> {
        self.0
            .get(name)
            .map(|builtin| *builtin)
    }
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn run(&self, _: &mut Context) -> Result<(), context::Error>;
}

macro_rules! impl_builtin {
    ($($ty:ident, $name:expr, $fn:ident, $input:ty => $output:ty;)*) => {
        fn builtins() -> Vec<&'static Builtin> {
            vec![
                $(&$ty,)*
            ]
        }

        $(
            pub struct $ty;

            impl Builtin for $ty {
                fn name(&self) -> &'static str {
                    $name
                }

                fn run(&self, context: &mut Context)
                    -> Result<(), context::Error>
                {
                    $fn(context)
                }
            }
        )*
    }
}

impl_builtin!(
    Print, "print",  print,  Expression => ();
    Eval,  "eval",   eval,   List => ();
    Define,"define", define, (List, List) => ();

    Add, "+", add, (Number, Number) => Number;
    Mul, "*", mul, (Number, Number) => Number;
);


fn print(context: &mut Context) -> Result<(), context::Error> {
    match context.stack().pop::<Expression>()? {
        Expression::Number(number) => print!("{}", number),
        Expression::List(_)        => unimplemented!(),
        Expression::String(string) => print!("{}", string),
        Expression::Word(_)        => unimplemented!(),
    }

    Ok(())
}

fn define(context: &mut Context) -> Result<(), context::Error> {
    let (body, name) = context.stack().pop::<(List, List)>()?;

    assert_eq!(name.0.len(), 1);
    let name = name.0.clone().pop().unwrap();

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

    context.define(name, body.clone());

    Ok(())
}

fn eval(context: &mut Context) -> Result<(), context::Error> {
    let list = context.stack().pop::<List>()?;
    context.evaluate(&mut list.into_iter())?;
    Ok(())
}

fn add(context: &mut Context) -> Result<(), context::Error> {
    let (a, b) = context.stack().pop::<(Number, Number)>()?;
    context.stack().push(a + b);
    Ok(())
}

fn mul(context: &mut Context) -> Result<(), context::Error> {
    let (a, b) = context.stack().pop::<(Number, Number)>()?;
    context.stack().push(a * b);
    Ok(())
}
