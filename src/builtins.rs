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
    tokenizer::Span,
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
        let data = f((self.0.data, self.1.data));
        let span = Span::merge(self.0.span, self.0.span);

        expression::Data { data, span }.into_expression()
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

    assert_eq!(name.data.0.len(), 1);
    let name = name.data.clone().0.pop().unwrap();

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

    context.define(name, body.data.clone());

    Ok(())
}

fn eval(context: &mut Context) -> Result {
    let list = context.stack().pop::<List>()?;
    context.evaluate(&mut list.data.into_iter())?;
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

    for item in list.data {
        context.stack().push::<Expression>(item);
        context.evaluate(&mut function.data.clone().into_iter())?;
    }

    let result = context.stack().destroy_substack();

    let span = Span::merge(list.span, function.span);
    let data = expression::Data {
        data: List(result),
        span,
    };
    context.stack().push::<List>(data);

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
