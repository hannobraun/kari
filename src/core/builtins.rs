use std::{
    collections::HashMap,
    result::Result as StdResult,
    vec,
};

use crate::core::{
    context::{
        self,
        Context,
    },
    expression::{
        self,
        Bool,
        Expression,
        Into as _,
        List,
        Number,
    },
    span::Span,
};


pub struct Builtins(HashMap<&'static str, &'static Builtin>);

impl Builtins {
    pub fn new() -> Self {
        let mut b = HashMap::new();

        for (name, builtin) in builtins() {
            b.insert(name, builtin);
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
    fn run(&self, operator: Span, _: &mut Context) -> Result;
}

macro_rules! impl_builtin {
    ($($ty:ident, $name:expr, $fn:ident, $input:ty => $output:ty;)*) => {
        fn builtins() -> Vec<(&'static str, &'static Builtin)> {
            vec![
                $(($name, &$ty),)*
            ]
        }

        $(
            pub struct $ty;

            impl Builtin for $ty {
                fn run(&self, operator: Span, context: &mut Context)
                    -> Result
                {
                    $fn(operator, context)
                }
            }
        )*
    }
}

impl_builtin!(
    Print,  "print",  print,  Expression => ();
    Define, "define", define, (List, List) => ();
    Fail,   "fail",   fail,   () => ();
    Eval,   "eval",   eval,   List => ();

    Drop, "drop", drop, Expression => ();
    Dup,  "dup",  dup,  Expression => (Expression, Expression);

    If,   "if",   r#if, (List, List) => ();
    Each, "each", each, (List, List) => List;

    Add, "+",   add, (Number, Number) => Number;
    Mul, "*",   mul, (Number, Number) => Number;
    Eq,  "=",   eq,  (Number, Number) => Bool;
    Gt,  ">",   gt,  (Number, Number) => Bool;
    Not, "not", not, Bool => Bool;
);


pub trait Compute : Sized {
    type Input;

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            expression::Data<R>: expression::Into;

}

impl<T> Compute for expression::Data<T> {
    type Input = T;

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            expression::Data<R>: expression::Into
    {
        let data = f(self.data);
        let span = operator.merge(self.span);

        expression::Data { data, span }.into_expression()
    }
}

impl<A, B> Compute for (expression::Data<A>, expression::Data<B>) {
    type Input = (A, B);

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            expression::Data<R>: expression::Into,
    {
        let data = f((self.0.data, self.1.data));
        let span = operator.merge(self.0.span).merge(self.1.span);

        expression::Data { data, span }.into_expression()
    }
}


pub type Result = StdResult<(), context::Error>;


fn print(operator: Span, context: &mut Context) -> Result {
    let expression = context.stack().pop::<Expression>(operator)?;
    print!("{}", expression.kind);

    Ok(())
}

fn define(operator: Span, context: &mut Context) -> Result {
    let (body, name) = context.stack().pop::<(List, List)>(operator)?;

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

fn fail(operator: Span, _: &mut Context) -> Result {
    Err(context::Error::Failure { operator })
}

fn eval(operator: Span, context: &mut Context) -> Result {
    let list = context.stack().pop::<List>(operator)?;
    context.evaluate(
        Some(operator),
        &mut list.data.into_iter(),
    )?;
    Ok(())
}


fn drop(operator: Span, context: &mut Context) -> Result {
    context.stack().pop::<Expression>(operator)?;
    Ok(())
}

fn dup(operator: Span, context: &mut Context) -> Result {
    let mut expression = context.stack().pop::<Expression>(operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push::<Expression>(expression.clone());
    context.stack().push::<Expression>(expression);

    Ok(())
}


fn each(operator: Span, context: &mut Context) -> Result {
    let (list, function) = context.stack().pop::<(List, List)>(operator)?;

    context.stack().create_substack();

    for item in list.data {
        context.stack().push::<Expression>(item);
        context.evaluate(
            Some(operator),
            &mut function.data.clone().into_iter(),
        )?;
    }

    let result = context.stack().destroy_substack();

    let data = expression::Data {
        data: List(result),
        span: operator.merge(list.span).merge(function.span),
    };
    context.stack().push::<List>(data);

    Ok(())
}


fn r#if(operator: Span, context: &mut Context) -> Result {
    let (function, condition) = context.stack().pop::<(List, List)>(operator)?;

    context.evaluate(Some(operator), &mut condition.data.into_iter())?;
    if context.stack().pop::<Bool>(operator)?.data.0 {
        context.evaluate(
            Some(operator),
            &mut function.data.into_iter(),
        )?;
    }

    Ok(())
}


fn add(operator: Span, context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>(operator)?
        .compute(operator, |(a, b)| a + b);
    context.stack().push_raw(result);
    Ok(())
}

fn mul(operator: Span, context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>(operator)?
        .compute(operator, |(a, b)| a * b);
    context.stack().push_raw(result);
    Ok(())
}

fn eq(operator: Span, context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>(operator)?
        .compute(operator, |(a, b)| Bool(a == b));
    context.stack().push_raw(result);
    Ok(())
}

fn gt(operator: Span, context: &mut Context) -> Result {
    let result = context
        .stack().pop::<(Number, Number)>(operator)?
        .compute(operator, |(a, b)| Bool(a > b));
    context.stack().push_raw(result);
    Ok(())
}

fn not(operator: Span, context: &mut Context) -> Result {
    let result = context
        .stack().pop::<Bool>(operator)?
        .compute(operator, |b| !b);
    context.stack().push_raw(result);
    Ok(())
}
