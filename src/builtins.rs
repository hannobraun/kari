use crate::{
    context::{
        self,
        Context,
    },
    function::Function,
    functions::{
        Functions,
        Scope,
    },
    types as t,
    value::{
        self,
        Compute as _,
        Value as _,
    },
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
    "print",   print,    (t::Any,);
    "define",  define,   (t::List, t::Symbol,);
    "define",  define_s, (t::List, t::Symbol, t::Scope,);
    "caller",  caller,   ();
    "fail",    fail,     ();
    "eval",    eval,     (t::List,);
    "load",    load,     (t::String,);
    "to_list", to_list,  (t::Symbol,);

    "drop",  drop,  (t::Any,);
    "clone", clone, (t::Any,);
    "swap",  swap,  (t::Any, t::Any,);
    "dig",   dig,   (t::Any, t::List,);

    "if", r#if, (t::List, t::List,);

    "list",    list,    (t::Number,);
    "map",     map,     (t::List, t::List,);
    "wrap",    wrap,    (t::Any,);
    "unwrap",  unwrap,  (t::List,);
    "prepend", prepend, (t::List, t::Any,);
    "append",  append,  (t::List, t::Any,);

    "+", add_n, (t::Number, t::Number,);
    "-", sub_n, (t::Number, t::Number,);
    "*", mul_n, (t::Number, t::Number,);
    "/", div_n, (t::Number, t::Number,);
    ">", gt_n,  (t::Number, t::Number,);

    "+", add_f, (t::Float, t::Float,);
    "-", sub_f, (t::Float, t::Float,);
    "*", mul_f, (t::Float, t::Float,);
    ">", gt_f,  (t::Float, t::Float,);

    "=",   eq,  (t::Any, t::Any,);
    "not", not, (t::Bool,);
);


fn print<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let expression = context.stack().pop(&t::Any);
    write!(context.output(), "{}", expression.kind)?;

    Ok(())
}

fn define<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let (body, name) = context.stack()
        .pop((&t::List, &t::Symbol));

    context.functions().define(
        scope,
        name.inner,
        &[],
        Function::UserDefined { body },
    )?;

    Ok(())
}

fn define_s<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (body, name, scope) = context.stack()
        .pop((&t::List, &t::Symbol, &t::Scope));

    context.functions().define(
        scope.inner,
        name.inner,
        &[],
        Function::UserDefined { body },
    )?;

    Ok(())
}

fn caller<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let caller = match context.call_stack().caller() {
        Some(caller) => caller.clone(),
        None         => return Err(context::Error::Caller),
    };

    context.stack().push(value::Scope::new(caller.scope, caller.span));

    Ok(())
}

fn fail<Host>(
    _: &mut Host,
    _: &mut dyn Context<Host>,
    _: Scope,
)
    -> Result
{
    Err(context::Error::Failure)
}

fn eval<Host>(
    host:    &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let list = context.stack().pop(&t::List);
    let span = context.call_stack().operator().clone().span.merge(&list.span);

    context.stack().create_substack();

    context.evaluate_list(
        host,
        list,
    )?;

    let items = context.stack().destroy_substack();

    let list = value::List::new(
        value::ListInner::from_values(
            items,
            context.functions().new_scope(scope, "list"),
        ),
        span,
    );
    context.stack().push(list);

    Ok(())
}

fn load<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let path = context.stack().pop(&t::String);

    let list = context.load(path, scope)?;
    context.stack().push(list);
    Ok(())
}

fn to_list<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let symbol = context.stack().pop(&t::Symbol);

    let (word, span) = symbol.open();
    let list_span    = context.call_stack().operator().span.clone().merge(&span);
    let word         = value::Any::new(value::Kind::Word(word), span);

    let list = value::List::new(
        value::ListInner::from_values(
            vec![word],
            context.functions().new_scope(scope, "list"),
        ),
        list_span,
    );
    context.stack().push(list);

    Ok(())
}


fn drop<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    context.stack().pop(&t::Any);
    Ok(())
}

fn clone<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let mut expression = context.stack().pop(&t::Any);

    expression.span = context.call_stack().operator().span.clone().merge(&expression.span);

    context.stack().push((expression.clone(), expression));

    Ok(())
}

fn swap<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (a, b) = context.stack().pop((&t::Any, &t::Any));
    context.stack().push((b, a));

    Ok(())
}

fn dig<Host>(
    host:    &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (item, f) = context.stack().pop((&t::Any, &t::List));
    context.evaluate_list(host, f)?;
    context.stack().push(item);
    Ok(())
}


fn r#if<Host>(
    host:    &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (function, condition) = context.stack()
        .pop((&t::List, &t::List));

    context.evaluate_list(host, condition)?;

    let evaluated_condition = context.stack().pop(&t::Bool);

    if evaluated_condition.inner {
        context.evaluate_list(
            host,
            function,
        )?;
    }

    Ok(())
}


fn list<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let len = context.stack()
        .pop(&t::Number);

    let mut items = Vec::new();
    let mut span  = len.span;

    for _ in 0 .. len.inner {
        let item = context.stack()
            .pop(&t::Any);

        span = span.merge(&item.span);
        items.insert(0, item);
    }

    let list = value::List::new(
        value::ListInner::from_values(
            items,
            context.functions().new_scope(scope, "list"),
        ),
        span,
    );

    context.stack().push(list);

    Ok(())
}

fn map<Host>(
    host:    &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (list, function) = context.stack()
        .pop((&t::List, &t::List));

    context.stack().create_substack();

    for item in list.inner.items {
        context.stack().push(item);
        context.evaluate_list(
            host,
            function.clone(),
        )?;
    }

    let result = context.stack().destroy_substack();

    let data = value::List::new(
        value::ListInner::from_values(
            result,
            context.functions().new_scope(list.inner.scope, "list"),
        ),
        context.call_stack().operator().span.clone().merge(&list.span).merge(&function.span),
    );
    context.stack().push(data);

    Ok(())
}

fn wrap<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    scope:   Scope,
)
    -> Result
{
    let arg = context.stack().pop(&t::Any);

    let span = context.call_stack().operator().span.clone().merge(&arg.span);
    let list = value::List::new(
        value::ListInner::from_values(
            vec![arg],
            context.functions().new_scope(scope, "list"),
        ),
        span,
    );

    context.stack().push(list);

    Ok(())
}

fn unwrap<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let list = context.stack().pop(&t::List);

    for value in list.inner.items {
        context.stack().push(value);
    }

    Ok(())
}

fn prepend<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (mut list, arg) = context.stack().pop((&t::List, &t::Any));

    list.span = context.call_stack().operator().span.clone().merge(&list.span).merge(&arg.span);
    list.inner.items.insert(0, arg);

    context.stack().push(list);

    Ok(())
}

fn append<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let (mut list, arg) = context.stack().pop((&t::List, &t::Any));

    list.span = context.call_stack().operator().span.clone().merge(&list.span).merge(&arg.span);
    list.inner.items.push(arg);

    context.stack().push(list);

    Ok(())
}


fn add_n<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Number, &t::Number))
        .compute::<value::Number, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn sub_n<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let difference = context.stack()
        .pop((&t::Number, &t::Number))
        .compute::<value::Number, _, _>(|(a, b)| a - b);

    context.stack().push(difference);

    Ok(())
}

fn mul_n<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Number, &t::Number))
        .compute::<value::Number, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn div_n<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let quotient = context.stack()
        .pop((&t::Number, &t::Number))
        .compute::<value::Number, _, _>(|(a, b)| a / b);

    context.stack().push(quotient);

    Ok(())
}

fn gt_n<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Number, &t::Number))
        .compute::<value::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn add_f<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Float, &t::Float))
        .compute::<value::Float, _, _>(|(a, b)| a + b);

    context.stack().push(sum);

    Ok(())
}

fn sub_f<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let sum = context.stack()
        .pop((&t::Float, &t::Float))
        .compute::<value::Float, _, _>(|(a, b)| a - b);

    context.stack().push(sum);

    Ok(())
}

fn mul_f<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let product = context.stack()
        .pop((&t::Float, &t::Float))
        .compute::<value::Float, _, _>(|(a, b)| a * b);

    context.stack().push(product);

    Ok(())
}

fn gt_f<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let is_greater = context.stack()
        .pop((&t::Float, &t::Float))
        .compute::<value::Bool, _, _>(|(a, b)| a > b);

    context.stack().push(is_greater);

    Ok(())
}


fn eq<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let is_equal = context.stack()
        .pop((&t::Any, &t::Any))
        .compute::<value::Bool, _, _>(|(a, b)| a == b);

    context.stack().push(is_equal);

    Ok(())
}

fn not<Host>(
    _:       &mut Host,
    context: &mut dyn Context<Host>,
    _:       Scope,
)
    -> Result
{
    let inverted = context.stack()
        .pop(&t::Bool)
        .compute::<value::Bool, _, _>(|b| !b);

    context.stack().push(inverted);

    Ok(())
}
