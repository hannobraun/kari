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
        Function,
        Host,
        Scope,
    },
};


pub type Result  = std::result::Result<(), context::Error>;


macro_rules! builtins {
    ($($name:expr, $fn:ident, ($($arg:expr,)*);)*) => {
        pub fn builtins<H>(builtins: &mut Scope<Function<H>>) {
            builtins
                $(
                    .define(
                        String::from($name),
                        &[$(&$arg,)*],
                        Function::Builtin($fn),
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


fn print<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let expression = context.stack().pop(&t::Any, &operator)?;
    write!(context.output(), "{}", expression.kind)?;

    Ok(())
}

fn define<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    scope:    &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let (body, name) = context.stack()
        .pop((&t::List, &t::Symbol), &operator)?;

    scope.define(name.inner, &[], Function::UserDefined(body))?;

    Ok(())
}

fn fail<H>(
    _:        Host<H>,
    _:        &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    Err(context::Error::Failure { operator })
}

fn eval<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    scope:    &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let list = context.stack().pop(&t::List, &operator)?;

    context.evaluate_list(
        scope,
        Some(operator),
        list,
    )?;
    Ok(())
}

fn load<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let path = context.stack().pop(&t::String, &operator)?;

    let list = context.load(path)?;
    context.stack().push(list);
    Ok(())
}


fn drop<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    context.stack().pop(&t::Any, &operator)?;
    Ok(())
}

fn dup<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let mut expression = context.stack().pop(&t::Any, &operator)?;

    expression.span = operator.merge(expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}


fn map<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    scope:    &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let (list, function) = context.stack()
        .pop((&t::List, &t::List), &operator)?;

    context.stack().create_substack();

    for item in list.inner {
        context.stack().push(item);
        context.evaluate_list(
            scope,
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


fn r#if<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    scope:    &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let (function, condition)  =context.stack()
        .pop((&t::List, &t::List), &operator)?;

    context.evaluate_list(scope, Some(operator.clone()), condition)?;

    let evaluated_condition = context.stack().pop(&t::Bool, &operator)?;

    if evaluated_condition.inner {
        context.evaluate_list(
            scope,
            Some(operator),
            function,
        )?;
    }

    Ok(())
}


fn add_n<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_n<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Number, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_n<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn add_f<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Float, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_f<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Float, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_f<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn eq<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let is_equal = context.stack()
        .pop((&t::Any, &t::Any), &operator)?
        .compute::<expr::Bool, _, _>(|(a, b)| a == b);

    context.stack().push(is_equal);

    Ok(())
}

fn not<H>(
    _:        Host<H>,
    context:  &mut dyn Context<H>,
    _:        &mut Scope<Function<H>>,
    operator: Span,
)
    -> Result
{
    let inverted = context.stack()
        .pop(&t::Bool, &operator)?
        .compute::<expr::Bool, _, _>(|b| !b);

    context.stack().push(inverted);

    Ok(())
}
