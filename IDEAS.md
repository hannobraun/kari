# Ideas

## Design

### Type Annotations

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

There's one advantage of the comment approach though: Thanks to syntax highlighting, the annotations are visually distinct. It might make sense to use a special syntax here, to preserve that attribute. Maybe create a special list syntax that creates a list of types on the stack, then executes a functions the checks said list against the stack. Maybe something like this:

``` kari
[: symbol :] # verifies that a symbol is on top of the stack
```

This would just be syntax sugar for the following:

``` kari
[ symbol ] annotation
```

### Overloading via Type Inference

It is highly desirable to support function overloading of user-defined functions. It would be nice to use type inference to determine the signatures, then there wouldn't need to be explicit type annotations in many cases.

Even in cases where type annotations are necessary or desirable, type inference would enable the use of inline type annotations for this. Then there wouldn't need to be a special syntax for function types.

### Function Types

Currently, each list has an associated scope, as each list could be a function. This is unnecessary from a design perspective, and makes some things slightly more complicated from an implementation perspective. Maybe it would be nicer to distinguish functions and list. A list would be just a list, while a function would be a composite type that has a list and a scope.

It would be possible to define functions like this:

``` kari
[ a b c ] fn
```

This would put an anonymous function on the stack that could then be `define`d as usual. It might make sense to make functions more visually distinct (also through syntax highlighting) by creating a special syntax for this:

```
[! a b c !]
```


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
