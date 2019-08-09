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
        expr::{
            self,
            Expression,
        },
        span::Span,
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


fn print(operator: Span, context: &mut Context) -> Result {
    let expression = context.stack().pop::<Expression>(&operator)?;
    print!("{}", expression.kind);

    Ok(())
}

fn define(operator: Span, context: &mut Context) -> Result {
    let (body, name) = context.stack()
        .pop::<(expr::List, expr::List)>(&operator)?;

    assert_eq!(name.inner.len(), 1);
    let name = name.clone().inner.pop().unwrap()
        .check::<expr::Word>()?;

    context.define(name, body.clone());

    Ok(())
}

fn fail(operator: Span, _: &mut Context) -> Result {
    Err(context::Error::Failure { operator })
}

fn eval(operator: Span, context: &mut Context) -> Result {
    let list = context.stack().pop::<expr::List>(&operator)?;

    context.evaluate(
        Some(operator),
        &mut list.into_iter(),
    )?;
    Ok(())
}

fn load(operator: Span, context: &mut Context) -> Result {
    let path = context.stack().pop::<expr::String>(&operator)?;

    let list = context.load(path)?;
    context.stack().push(list);
    Ok(())
}


fn drop(operator: Span, context: &mut Context) -> Result {
    context.stack().pop::<Expression>(&operator)?;
    Ok(())
}

fn dup(operator: Span, context: &mut Context) -> Result {
    let mut expression = context.stack().pop::<Expression>(&operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}


fn each(operator: Span, context: &mut Context) -> Result {
    let (list, function) = context.stack()
        .pop::<(expr::List, expr::List)>(&operator)?;

    context.stack().create_substack();

    for item in list.inner {
        context.stack().push(item);
        context.evaluate(
            Some(operator.clone()),
            &mut function.clone().into_iter(),
        )?;
    }

    let result = context.stack().destroy_substack();

    let data = expr::List::new(
        result,
        operator.merge(list.span).merge(function.span),
    );
    context.stack().push(data);

    Ok(())
}


fn r#if(operator: Span, context: &mut Context) -> Result {
    let (function, condition)  =context.stack()
        .pop::<(expr::List, expr::List)>(&operator)?;

    context.evaluate(Some(operator.clone()), &mut condition.into_iter())?;

    let evaluated_condition = context.stack().pop::<expr::Bool>(&operator)?;

    if evaluated_condition.inner {
        context.evaluate(
            Some(operator),
            &mut function.into_iter(),
        )?;
    }

    Ok(())
}


fn add(operator: Span, context: &mut Context) -> Result {
    let (a, b) = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?;

    context.stack().push(a + b);
    Ok(())
}

fn mul(operator: Span, context: &mut Context) -> Result {
    let (a, b) = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?;

    context.stack().push(a * b);
    Ok(())
}

fn eq(operator: Span, context: &mut Context) -> Result {
    let (a, b) = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?;

    let result = expr::Bool::new(
        a == b,
        a.span.merge(b.span),
    );

    context.stack().push(result);
    Ok(())
}

fn gt(operator: Span, context: &mut Context) -> Result {
    let (a, b) = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?;

    let result = expr::Bool::new(
        a > b,
        a.span.merge(b.span),
    );

    context.stack().push(result);
    Ok(())
}

fn not(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop::<expr::Bool>(&operator)?;
    context.stack().push(!b);
    Ok(())
}
