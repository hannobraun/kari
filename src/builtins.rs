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
    expression::{
        self,
        Expression,
        Into as _,
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

    Drop, "drop", drop, Expression => ();
    Dup,  "dup",  dup,  Expression => (Expression, Expression);

    Each, "each", each, (List, List) => List;

    Add, "+", add, (Number, Number) => Number;
    Mul, "*", mul, (Number, Number) => Number;
);


pub trait Compute : Sized {
    type Input;

    fn compute<F, R>(self, _: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            expression::Data<R>: expression::Into;

}

impl<A, B> Compute for (expression::Data<A>, expression::Data<B>)
{
    type Input = (A, B);

    fn compute<F, R>(self, f: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            expression::Data<R>: expression::Into,
    {
        let r = f(((self.0).0, (self.1).0));
        expression::Data(r).into_expression()
    }
}


pub type Result = StdResult<(), context::Error>;


fn print(context: &mut Context) -> Result {
    let expression = context.stack().pop::<Expression>()?;
    print!("{}", expression.kind);

    Ok(())
}

fn define(context: &mut Context) -> Result {
    let (body, name) = context.stack().pop::<(List, List)>()?;

    assert_eq!((name.0).0.len(), 1);
    let name = name.0.clone().0.pop().unwrap();

    let name = match name.kind {
        expression::Kind::Word(word) => {
            word
        }
        kind => {
            panic!(
                "Unexpected expression: {:?}\n",
                kind,
            );
        }
    };

    context.define(name, body.0.clone());

    Ok(())
}

fn eval(context: &mut Context) -> Result {
    let list = context.stack().pop::<List>()?.0;
    context.evaluate(&mut list.into_iter())?;
    Ok(())
}


fn drop(context: &mut Context) -> Result {
    context.stack().pop::<Expression>()?;
    Ok(())
}

fn dup(context: &mut Context) -> Result {
    let expression = context.stack().pop::<Expression>()?;

    context.stack().push::<Expression>(expression.clone());
    context.stack().push::<Expression>(expression);

    Ok(())
}


fn each(context: &mut Context) -> Result {
    let (list, function) = context.stack().pop::<(List, List)>()?;

    context.stack().create_substack();

    for item in list.0 {
        context.stack().push::<Expression>(item);
        context.evaluate(&mut function.0.clone().into_iter())?;
    }

    let list = context.stack().destroy_substack();
    context.stack().push::<List>(expression::Data(List(list)));

    Ok(())
}


fn add(context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>()?
        .compute(|(a, b)| a + b);
    context.stack().push_raw(result);
    Ok(())
}

fn mul(context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>()?
        .compute(|(a, b)| a * b);
    context.stack().push_raw(result);
    Ok(())
}
