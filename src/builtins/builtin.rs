use std::{
    result::Result as StdResult,
    vec,
};

use crate::{
    builtins::context::{
        self,
        Context,
    },
    data::{
        expression::{
            self,
            Bool,
            Check as _,
            Expression,
            Into as _,
            List,
            Number,
        },
        span::{
            Span,
            WithSpan,
        },
    },
};


pub type Builtin = fn(Span, &mut Context) -> Result;
pub type Result  = StdResult<(), context::Error>;


macro_rules! builtins {
    ($($name:expr, $fn:ident;)*) => {
        pub fn builtins() -> Vec<(&'static str, Builtin)> {
            vec![
                $(($name, $fn),)*
            ]
        }
    }
}

builtins!(
    "print",  print;
    "define", define;
    "fail",   fail;
    "eval",   eval;
    "load",   load;

    "drop", drop;
    "dup",  dup;

    "if",   r#if;
    "each", each;

    "+",   add;
    "*",   mul;
    "=",   eq;
    ">",   gt;
    "not", not;
);


pub trait Compute : Sized {
    type Input;

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F: Fn(Self::Input) -> R,
            WithSpan<R>: expression::Into;

}

impl<T> Compute for WithSpan<T> {
    type Input = T;

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F:           Fn(Self::Input) -> R,
            WithSpan<R>: expression::Into
    {
        let value = f(self.value);
        let span  = operator.merge(self.span);

        WithSpan { value, span }.into_expression()
    }
}

impl<A, B> Compute for (WithSpan<A>, WithSpan<B>) {
    type Input = (A, B);

    fn compute<F, R>(self, operator: Span, f: F) -> Expression
        where
            F:           Fn(Self::Input) -> R,
            WithSpan<R>: expression::Into,
    {
        let value = f((self.0.value, self.1.value));
        let span  = operator.merge(self.0.span).merge(self.1.span);

        WithSpan { value, span }.into_expression()
    }
}


fn print(operator: Span, context: &mut Context) -> Result {
    let expression = context.stack().pop_raw(&operator)?;
    print!("{}", expression.kind);

    Ok(())
}

fn define(operator: Span, context: &mut Context) -> Result {
    let name = context.stack().pop_raw(&operator)?;
    let body = context.stack().pop_raw(&operator)?;

    let (body, name): (WithSpan<List>, WithSpan<List>) = (body, name).check()?;

    assert_eq!(name.value.0.len(), 1);
    let name = name.value.clone().0.pop().unwrap();

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

    context.define(name, body.value.clone());

    Ok(())
}

fn fail(operator: Span, _: &mut Context) -> Result {
    Err(context::Error::Failure { operator })
}

fn eval(operator: Span, context: &mut Context) -> Result {
    let list: WithSpan<List> = context.stack().pop_raw(&operator)?.check()?;
    context.evaluate(
        Some(operator),
        &mut list.value.into_iter(),
    )?;
    Ok(())
}

fn load(operator: Span, context: &mut Context) -> Result {
    let path: WithSpan<String> = context.stack().pop_raw(&operator)?.check()?;
    let list = context.load(path.value)?;
    context.stack().push(list);
    Ok(())
}


fn drop(operator: Span, context: &mut Context) -> Result {
    context.stack().pop_raw(&operator)?;
    Ok(())
}

fn dup(operator: Span, context: &mut Context) -> Result {
    let mut expression = context.stack().pop_raw(&operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}


fn each(operator: Span, context: &mut Context) -> Result {
    let function = context.stack().pop_raw(&operator)?;
    let list     = context.stack().pop_raw(&operator)?;

    let (list, function): (WithSpan<List>, WithSpan<List>) =
        (list, function).check()?;

    context.stack().create_substack();

    for item in list.value {
        context.stack().push(item);
        context.evaluate(
            Some(operator.clone()),
            &mut function.value.clone().into_iter(),
        )?;
    }

    let result = context.stack().destroy_substack();

    let data = WithSpan {
        value: List(result),
        span:  operator.merge(list.span).merge(function.span),
    };
    context.stack().push(data);

    Ok(())
}


fn r#if(operator: Span, context: &mut Context) -> Result {
    let condition = context.stack().pop_raw(&operator)?;
    let function  = context.stack().pop_raw(&operator)?;

    let (function, condition): (WithSpan<List>, WithSpan<List>) =
        (function, condition).check()?;

    context.evaluate(Some(operator.clone()), &mut condition.value.into_iter())?;
    if context.stack().pop::<Bool>(&operator)?.value.0 {
        context.evaluate(
            Some(operator),
            &mut function.value.into_iter(),
        )?;
    }

    Ok(())
}


fn add(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let result = (a, b)
        .check()?
        .compute(operator, |(a, b): (Number, Number)| a + b);

    context.stack().push(result);
    Ok(())
}

fn mul(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let result = (a, b)
        .check()?
        .compute(operator, |(a, b): (Number, Number)| a * b);

    context.stack().push(result);
    Ok(())
}

fn eq(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let result = (a, b)
        .check()?
        .compute(operator, |(a, b): (Number, Number)| Bool(a == b));

    context.stack().push(result);
    Ok(())
}

fn gt(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let result = (a, b)
        .check()?
        .compute(operator, |(a, b): (Number, Number)| Bool(a > b));

    context.stack().push(result);
    Ok(())
}

fn not(operator: Span, context: &mut Context) -> Result {
    let b: WithSpan<Bool> = context.stack().pop_raw(&operator)?.check()?;

    let result = b.compute(operator, |b: Bool| !b);

    context.stack().push(result);
    Ok(())
}
