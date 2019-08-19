use std::collections::HashMap;

use crate::{
    context::{
        self,
        Context,
    },
    data::{
        expr::{
            self,
            Compute as _,
            Expr as _,
        },
        span::Span,
    },
};


pub struct Builtins(HashMap<&'static str, Builtin>);

impl Builtins {
    pub fn new() -> Self {
        let mut b = HashMap::new();

        for (name, builtin) in builtins() {
            b.insert(name, builtin);
        }

        Self(b)
    }

    pub fn get(&self, name: &str) -> Option<Builtin> {
        self.0
            .get(name)
            .map(|builtin| *builtin)
    }
}


pub type Builtin = fn(&mut dyn Context, Span) -> Result;
pub type Result  = std::result::Result<(), context::Error>;


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

    "if",  r#if;
    "map", map;

    "+",   add;
    "*",   mul;
    "=",   eq;
    ">",   gt;
    "not", not;
);


fn print(context: &mut dyn Context, operator: Span) -> Result {
    let expression = context.stack().pop::<expr::Any>(&operator)?;
    write!(context.output(), "{}", expression.kind)?;

    Ok(())
}

fn define(context: &mut dyn Context, operator: Span) -> Result {
    let (body, name) = context.stack()
        .pop::<(expr::List, expr::Symbol)>(&operator)?;

    context.define(name, body.clone());

    Ok(())
}

fn fail(_: &mut dyn Context, operator: Span) -> Result {
    Err(context::Error::Failure { operator })
}

fn eval(context: &mut dyn Context, operator: Span) -> Result {
    let list = context.stack().pop::<expr::List>(&operator)?;

    context.evaluate(
        Some(operator),
        &mut list.into_iter(),
    )?;
    Ok(())
}

fn load(context: &mut dyn Context, operator: Span) -> Result {
    let path = context.stack().pop::<expr::String>(&operator)?;

    let list = context.load(path)?;
    context.stack().push(list);
    Ok(())
}


fn drop(context: &mut dyn Context, operator: Span) -> Result {
    context.stack().pop::<expr::Any>(&operator)?;
    Ok(())
}

fn dup(context: &mut dyn Context, operator: Span) -> Result {
    let mut expression = context.stack().pop::<expr::Any>(&operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}


fn map(context: &mut dyn Context, operator: Span) -> Result {
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


fn r#if(context: &mut dyn Context, operator: Span) -> Result {
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


fn add(context: &mut dyn Context, operator: Span) -> Result {
    let sum = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul(context: &mut dyn Context, operator: Span) -> Result {
    let product = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn eq(context: &mut dyn Context, operator: Span) -> Result {
    let is_equal = context.stack()
        .pop::<(expr::Any, expr::Any)>(&operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a == b);

    context.stack().push(is_equal);

    Ok(())
}

fn gt(context: &mut dyn Context, operator: Span) -> Result {
    let is_greater = context.stack()
        .pop::<(expr::Number, expr::Number)>(&operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}

fn not(context: &mut dyn Context, operator: Span) -> Result {
    let inverted = context.stack()
        .pop::<expr::Bool>(&operator)?
        .compute::<expr::Bool, _, _>(|b| !b);

    context.stack().push(inverted);

    Ok(())
}
