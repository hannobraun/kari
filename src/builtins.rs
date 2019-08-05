use std::{
    collections::HashMap,
    result::Result as StdResult,
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

    pub fn builtin(&self, name: &str) -> Option<&'static Builtin> {
        self.0
            .get(name)
            .map(|builtin| *builtin)
    }
}


pub trait Builtin {
    fn name(&self) -> &'static str;
    fn run(&self, _: &mut Context) -> Result;
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
                    -> Result
                {
                    $fn(context)
                }
            }
        )*
    }
}

impl_builtin!(
    Print,  "print",  print,  Expression => ();
    Eval,   "eval",   eval,   List => ();
    Define, "define", define, (List, List) => ();

    Add, "+", add, (Number, Number) => Number;
    Mul, "*", mul, (Number, Number) => Number;
);


pub type Result = StdResult<(), context::Error>;


fn print(context: &mut Context) -> Result {
    let expression = context.stack().pop::<Expression>()?;
    print!("{}", expression);

    Ok(())
}

fn define(context: &mut Context) -> Result {
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

fn eval(context: &mut Context) -> Result {
    let list = context.stack().pop::<List>()?;
    context.evaluate(&mut list.into_iter())?;
    Ok(())
}

fn add(context: &mut Context) -> Result {
    let (a, b) = context.stack().pop::<(Number, Number)>()?;
    context.stack().push(Number(a.0 + b.0));
    Ok(())
}

fn mul(context: &mut Context) -> Result {
    let (a, b) = context.stack().pop::<(Number, Number)>()?;
    context.stack().push(Number(a.0 * b.0));
    Ok(())
}
