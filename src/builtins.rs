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
        types as t,
    },
    scope::{
        Builtin,
        Function,
        Scope,
    },
};


pub type Result  = std::result::Result<(), context::Error>;


macro_rules! builtins {
    ($($name:expr, $fn:ident, ($($arg:expr,)*);)*) => {
        pub fn builtins<Host>(builtins: &mut Scope<Function<Host>>) {
            builtins
                $(
                    .define(
                        String::from($name),
                        &[$(&$arg,)*],
                        Function::Builtin($fn as Builtin),
                    )
                    .expect("Failed to define builtin")
                )*;
        }
    }
}

builtins!(
    "print",  print,  (t::Any,);
    "define", define, (t::List, t::Symbol,);
    "fail",   fail,   ();
    "eval",   eval,   (t::List,);
    "load",   load,   (t::String,);

    "drop", drop, (t::Any,);
    "dup",  dup,  (t::Any,);

    "if",  r#if, (t::List, t::List,);
    "map", map,  (t::List, t::List,);

    "+", add_n, (t::Number, t::Number,);
    "*", mul_n, (t::Number, t::Number,);
    ">", gt_n,  (t::Number, t::Number,);

    "+", add_f, (t::Float, t::Float,);
    "*", mul_f, (t::Float, t::Float,);
    ">", gt_f,  (t::Float, t::Float,);

    "=",   eq,  (t::Any, t::Any,);
    "not", not, (t::Bool,);
);


fn print(context: &mut dyn Context, operator: Span) -> Result {
    let expression = context.stack().pop(&t::Any, &operator)?;
    write!(context.output(), "{}", expression.kind)?;

    Ok(())
}

fn define(context: &mut dyn Context, operator: Span) -> Result {
    let (body, name) = context.stack()
        .pop((&t::List, &t::Symbol), &operator)?;

    context.define(name, body.clone())?;

    Ok(())
}

fn fail(_: &mut dyn Context, operator: Span) -> Result {
    Err(context::Error::Failure { operator })
}

fn eval(context: &mut dyn Context, operator: Span) -> Result {
    let list = context.stack().pop(&t::List, &operator)?;

    context.evaluate_list(
        Some(operator),
        list,
    )?;
    Ok(())
}

fn load(context: &mut dyn Context, operator: Span) -> Result {
    let path = context.stack().pop(&t::String, &operator)?;

    let list = context.load(path)?;
    context.stack().push(list);
    Ok(())
}


fn drop(context: &mut dyn Context, operator: Span) -> Result {
    context.stack().pop(&t::Any, &operator)?;
    Ok(())
}

fn dup(context: &mut dyn Context, operator: Span) -> Result {
    let mut expression = context.stack().pop(&t::Any, &operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}


fn map(context: &mut dyn Context, operator: Span) -> Result {
    let (list, function) = context.stack()
        .pop((&t::List, &t::List), &operator)?;

    context.stack().create_substack();

    for item in list.inner {
        context.stack().push(item);
        context.evaluate_list(
            Some(operator.clone()),
            function.clone(),
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
        .pop((&t::List, &t::List), &operator)?;

    context.evaluate_list(Some(operator.clone()), condition)?;

    let evaluated_condition = context.stack().pop(&t::Bool, &operator)?;

    if evaluated_condition.inner {
        context.evaluate_list(
            Some(operator),
            function,
        )?;
    }

    Ok(())
}


fn add_n(context: &mut dyn Context, operator: Span) -> Result {
    let sum = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_n(context: &mut dyn Context, operator: Span) -> Result {
    let product = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_n(context: &mut dyn Context, operator: Span) -> Result {
    let is_greater = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn add_f(context: &mut dyn Context, operator: Span) -> Result {
    let sum = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Float, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_f(context: &mut dyn Context, operator: Span) -> Result {
    let product = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Float, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_f(context: &mut dyn Context, operator: Span) -> Result {
    let is_greater = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn eq(context: &mut dyn Context, operator: Span) -> Result {
    let is_equal = context.stack()
        .pop((&t::Any, &t::Any), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a == b);

    context.stack().push(is_equal);

    Ok(())
}

fn not(context: &mut dyn Context, operator: Span) -> Result {
    let inverted = context.stack()
        .pop(&t::Bool, &operator)?
        .compute::<expr::Bool, _, _>(|b| !b);

    context.stack().push(inverted);

    Ok(())
}
