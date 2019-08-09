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
        expr,
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
    let expression = context.stack().pop_raw(&operator)?;
    print!("{}", expression.kind);

    Ok(())
}

fn define(operator: Span, context: &mut Context) -> Result {
    let name = context.stack().pop_raw(&operator)?;
    let body = context.stack().pop_raw(&operator)?;

    let (body, name) = expr::E2(body, name).check::<expr::List, expr::List>()?;

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
    let list = context.stack().pop_raw(&operator)?.check::<expr::List>()?;
    context.evaluate(
        Some(operator),
        &mut list.into_iter(),
    )?;
    Ok(())
}

fn load(operator: Span, context: &mut Context) -> Result {
    let path: expr::String = context.stack()
        .pop_raw(&operator)?
        .check()?;
    let list = context.load(path)?;
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

    let (list, function) = expr::E2(list, function)
        .check::<expr::List, expr::List>()?;

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
    let condition = context.stack().pop_raw(&operator)?;
    let function  = context.stack().pop_raw(&operator)?;

    let (function, condition) = expr::E2(function, condition)
        .check::<expr::List, expr::List>()?;

    context.evaluate(Some(operator.clone()), &mut condition.into_iter())?;

    let evaluated_condition = context.stack()
        .pop_raw(&operator)?
        .check::<expr::Bool>()?;

    if evaluated_condition.inner {
        context.evaluate(
            Some(operator),
            &mut function.into_iter(),
        )?;
    }

    Ok(())
}


fn add(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let e = expr::E2(a, b)
        .check::<expr::Number, expr::Number>()?;

    context.stack().push(e.0 + e.1);
    Ok(())
}

fn mul(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let e = expr::E2(a, b)
        .check::<expr::Number, expr::Number>()?;

    context.stack().push(e.0 * e.1);
    Ok(())
}

fn eq(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let e = expr::E2(a, b)
        .check::<expr::Number, expr::Number>()?;

    let result = expr::Bool::new(
        e.0 == e.1,
        e.0.span.merge(e.1.span),
    );

    context.stack().push(result);
    Ok(())
}

fn gt(operator: Span, context: &mut Context) -> Result {
    let b = context.stack().pop_raw(&operator)?;
    let a = context.stack().pop_raw(&operator)?;

    let e = expr::E2(a, b)
        .check::<expr::Number, expr::Number>()?;

    let result = expr::Bool::new(
        e.0 > e.1,
        e.0.span.merge(e.1.span),
    );

    context.stack().push(result);
    Ok(())
}

fn not(operator: Span, context: &mut Context) -> Result {
    let b: expr::Bool = context.stack().pop_raw(&operator)?.check()?;
    context.stack().push(!b);
    Ok(())
}
