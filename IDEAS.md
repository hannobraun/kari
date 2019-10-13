# Ideas

## Design

### Assert Stack Types

I've found that more complicated code benefits from comments that explicitely point out which types are currently at the top of the stack. But of course, such comments can go out of date, or be wrong in the first place.

There should be a builtin that takes a list of types and fails, if those are not the types currently on the stack. So something like this:
```
# on stack: symbol
dup          # => symbol symbol
to_list      # => symbol [ word ]
swap wrap    # => [ word ] [ symbol ]
swap prepend # => [ [ word ] symbol ]
```

Would become something like this:
```
# on stack: symbol
[ symbol ] on_stack
dup          [ symbol symbol ] on_stack
to_list      [ symbol [ word ] ] on_stack
swap wrap    [ [ word ] [ symbol ] ] on_stack
swap prepend [ [ [ word ] symbol ] ] on_stack
```

There's one advantage of the comment approach though: Thanks to syntax highlighting, the annotations are visually distinct. It might make sense to use a special syntax here, to preserve that attribute.


## Implementation

### Builtin Input/Output

Builtins already need to define their input. It probably makes sense to make that directly available to them as an argument to the builtin function. This would be less work and less error-prone. It might even be possible to take stack manipulation out of them completely, if outputs are also defined at some point.

### `Evaluator` as `Stream` of side effects

If I ever end up modelling side effects somehow, then `Evaluator` could be turned into a stream of side effects that the runner pulls out of it.

### Error Messages

- Add spans to functions, so a type error for a function can point to where the function is defined.
- Make it possible to return multiple spans per error. For example the type error message can point to the expression that has the wrong type, as well as the operator that expects a different one.
- Give spans a type, so operands and operators can be colored differently in error messages.
- Extend the spans of expressions that are the result of an evaluation, so they also point to the operator that did the evaluation (like `eval`, `each`). This requires support for discontinuous spans, or maybe expressions can just have different spans.
- When a function definition conlicts with an existing definition, that definition should be shown.

### REPL

Add a REPL. As far as debugging tools go, this is probably the most bang for the buck.

Could it even be possible to emulate some debugging features by implementing a `step` builtin, which evaluates the first word of a list, but leaves the rest of the list?

Over time, it might be possible to make the language so powerful, that a debugger can just be implemented as a library that is used with the REPL.

### Pipeline

Extend the pipeline with more stages. Add a stage that converts expressions to values. Not sure if that's possible without full evaluation, as that stage would have to keep track of scopes, but it seems to be worth a try.

Maybe more features of the evaluator can be split out into simple pipeline stages.

### Define scope-less `define` in `std`

Once non-builtin functions support overloading, the scope-less `define` can be implemented in terms of the builtin, scope-taking `define`.
