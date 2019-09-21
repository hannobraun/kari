use crate::{
    context::{
        self,
        Context,
    },
    data::{
        functions::{
            Functions,
            Scope,
        },
        span::Span,
        types as t,
        value::{
            self,
            Compute as _,
            Value as _,
        },
    },
    function::Function,
};


pub type Result  = std::result::Result<(), context::Error>;


macro_rules! builtins {
    ($($name:expr, $fn:ident, ($($arg:expr,)*);)*) => {
        pub fn builtins<Host>(functions: &mut Functions<Function<Host>>) {
            let scope = functions.root_scope();

            functions
                $(
                    .define(
                        scope,
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
    "print",   print,   (t::Any,);
    "define",  define,  (t::List, t::Symbol,);
    "fail",    fail,    ();
    "eval",    eval,    (t::List,);
    "load",    load,    (t::String,);
    "to_list", to_list, (t::Symbol,);

    "drop", drop, (t::Any,);
    "dup",  dup,  (t::Any,);
    "swap", swap, (t::Any, t::Any,);

    "if", r#if, (t::List, t::List,);

    "map",     map,     (t::List, t::List,);
    "wrap",    wrap,    (t::Any,);
    "unwrap",  unwrap,  (t::List,);
    "prepend", prepend, (t::List, t::Any,);

    "+", add_n, (t::Number, t::Number,);
    "*", mul_n, (t::Number, t::Number,);
    "/", div_n, (t::Number, t::Number,);
    ">", gt_n,  (t::Number, t::Number,);

    "+", add_f, (t::Float, t::Float,);
    "*", mul_f, (t::Float, t::Float,);
    ">", gt_f,  (t::Float, t::Float,);

    "=",   eq,  (t::Any, t::Any,);
    "not", not, (t::Bool,);
);


fn print<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let expression = context.stack().pop(&t::Any, &operator)?;
    write!(context.output(), "{}", expression.kind)?;

    Ok(())
}

fn define<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let (body, name) = context.stack()
        .pop((&t::List, &t::Symbol), &operator)?;

    let scope = context.functions().root_scope();

    context.functions().define(
        scope,
        name.inner,
        &[],
        Function::UserDefined { body },
    )?;

    Ok(())
}

fn fail<Host>(
    _:        &mut Host,
    _:        &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    Err(context::Error::Failure { operator })
}

fn eval<Host>(
    host:     &mut Host,
    context:  &mut dyn Context<Host>,
    scope:    Scope,
    operator: Span,
)
    -> Result
{
    let list = context.stack().pop(&t::List, &operator)?;
    let span = operator.clone().merge(&list.span);

    context.stack().create_substack();

    context.evaluate_list(
        host,
        Some(operator),
        list,
    )?;

    let items = context.stack().destroy_substack();

    let list = value::List::new(
        value::ListInner::from_values(
            items,
            context.functions().new_scope(scope),
        ),
        span,
    );
    context.stack().push(list);

    Ok(())
}

fn load<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    scope:    Scope,
    operator: Span,
)
    -> Result
{
    let path = context.stack().pop(&t::String, &operator)?;

    let list = context.load(path, scope)?;
    context.stack().push(list);
    Ok(())
}

fn to_list<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    scope:    Scope,
    operator: Span,
)
    -> Result
{
    let symbol = context.stack().pop(&t::Symbol, &operator)?;

    let (word, span) = symbol.open();
    let list_span    = operator.merge(&span);
    let word         = value::Any::new(value::Kind::Word(word), span);

    let list = value::List::new(
        value::ListInner::from_values(
            vec![word],
            context.functions().new_scope(scope),
        ),
        list_span,
    );
    context.stack().push(list);

    Ok(())
}


fn drop<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    context.stack().pop(&t::Any, &operator)?;
    Ok(())
}

fn dup<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let mut expression = context.stack().pop(&t::Any, &operator)?;

    expression.span = operator.merge(&expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}

fn swap<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let (a, b) = context.stack().pop((&t::Any, &t::Any), &operator)?;
    context.stack().push((b, a));

    Ok(())
}


fn r#if<Host>(
    host:     &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let (function, condition)  =context.stack()
        .pop((&t::List, &t::List), &operator)?;

    context.evaluate_list(host, Some(operator.clone()), condition)?;

    let evaluated_condition = context.stack().pop(&t::Bool, &operator)?;

    if evaluated_condition.inner {
        context.evaluate_list(
            host,
            Some(operator),
            function,
        )?;
    }

    Ok(())
}


fn map<Host>(
    host:     &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let (list, function) = context.stack()
        .pop((&t::List, &t::List), &operator)?;

    context.stack().create_substack();

    for item in list.inner.items {
        context.stack().push(item);
        context.evaluate_list(
            host,
            Some(operator.clone()),
            function.clone(),
        )?;
    }

    let result = context.stack().destroy_substack();

    let data = value::List::new(
        value::ListInner::from_values(
            result,
            context.functions().new_scope(list.inner.scope),
        ),
        operator.merge(&list.span).merge(&function.span),
    );
    context.stack().push(data);

    Ok(())
}

fn wrap<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    scope:    Scope,
    operator: Span,
)
    -> Result
{
    let arg = context.stack().pop(&t::Any, &operator)?;

    let span = operator.merge(&arg.span);
    let list = value::List::new(
        value::ListInner::from_values(
            vec![arg],
            context.functions().new_scope(scope),
        ),
        span,
    );

    context.stack().push(list);

    Ok(())
}

fn unwrap<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let list = context.stack().pop(&t::List, &operator)?;

    for value in list.inner.items {
        context.stack().push(value);
    }

    Ok(())
}

fn prepend<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let (mut list, arg) = context.stack().pop((&t::List, &t::Any), &operator)?;

    list.span = operator.merge(&list.span).merge(&arg.span);
    list.inner.items.insert(0, arg);

    context.stack().push(list);

    Ok(())
}


fn add_n<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<value::Number, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_n<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<value::Number, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn div_n<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let quotient = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<value::Number, _, _>(|(a, b)| a / b);

    context.stack().push(quotient);

    Ok(())
}

fn gt_n<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Number, &t::Number), &operator)?
        .compute::<value::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn add_f<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<value::Float, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn mul_f<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<value::Float, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_f<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Float, &t::Float), &operator)?
        .compute::<value::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn eq<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let is_equal = context.stack()
        .pop((&t::Any, &t::Any), &operator)?
        .compute::<value::Bool, _, _>(|(a, b)| a == b);

    context.stack().push(is_equal);

    Ok(())
}

fn not<Host>(
    _:        &mut Host,
    context:  &mut dyn Context<Host>,
    _:        Scope,
    operator: Span,
)
    -> Result
{
    let inverted = context.stack()
        .pop(&t::Bool, &operator)?
        .compute::<value::Bool, _, _>(|b| !b);

    context.stack().push(inverted);

    Ok(())
}
