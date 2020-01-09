use std::io::{
    Cursor,
    stdout,
    stderr,
};

use kari::{
    prelude::*,
    context::Context,
    functions::Scope,
    value::{
        t,
        v,
    },
    interpreter::Interpreter,
};


fn main() {
    // Running a Kari program can result in output to stdout and stderr. We
    // could capture that output and use it for whatever we want, but here,
    // we'll just pipe it to this application's stdout and stderr.
    let stdout = Box::new(stdout());
    let stderr = Box::new(stderr());

    // Our Kari program. This could also be loaded from a file, input by the
    // user at runtime, or whatever else is appropriate for your application.
    // We need to wrap this in a `Cursor`, as a string or byte slice by itself
    // doesn't implement `io::Seek`, which is required by the interpreter.
    let program = Cursor::new("6 7 * is_42?");

    // The host is what provides an interface between the Kari interpreter and
    // the outside world. The host will be passed into all the builtin functions
    // your application defines.
    let mut host = Host;

    let stack = Interpreter::new(stdout, stderr)
        // The Kari language comes with a number of builtin functions. This will
        // make those available for your program. You could also not do this and
        // only make your own builtin functions available, or selectively
        // include builtin functions from the Kari interpreter.
        .with_default_builtins()
        // The prelude is a bit of Kari code that is run before every Kari
        // program. It provides the `import` function used to import modules.
        // You have the option to not include this for your own program.
        .with_default_prelude(&mut host)
            .expect("Error loading prelude")
        // Register a builtin function that will be made available to your Kari
        // program. You can call this method any number of times to register
        // more builtin functions.
        .with_builtin("is_42?", &[], is_42)
            .expect("Error registering builtin function")
        // We're done configuring the interpreter. It's time to run your
        // program. This requires the host, a string that is used in error
        // messages to refer to your program (this could be a file path, if we
        // loaded the program from a file), and of course your program.
        .run(&mut host, "<program>".into(), Box::new(program))
        .expect("Error running Kari program");

    print!("Result of the program:\n");
    for value in stack {
        print!("{}\n", value.kind);
    }
}


fn is_42(host: &mut Host, context: &mut dyn Context<Host>, _scope: Scope)
    -> kari::builtins::Result
{
    let is_42 = context.stack()
        .pop::<v::Any>()?
        .cast(t::Number)?
        .compute::<v::Bool, _, _>(|value| host.is_42(value));
    context.stack().push(is_42);
    Ok(())
}


struct Host;

impl Host {
    /// This is a method that will be callable by the builtins you define
    ///
    /// This is obviously a nonsensical example, but this method could do
    /// anything: File I/O, network I/O, or any other capabilities else you want
    /// to make available to your Kari program.
    fn is_42(&self, value: u32) -> bool {
        value == 42
    }
}
