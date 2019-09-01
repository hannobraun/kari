# Ideas

## Design

### Function Shadowing

At the moment, user-defined functions shadow extensions, which shadow builtins. It might be better to disallow shadowing altogether. Or maybe it's better to defer the decision until scopes are in the language.


## Implementation

### `Evaluator` as `Stream` of side effects

If I ever end up modelling side effects somehow, then `Evaluator` could be turned into a stream of side effects that the runner pulls out of it.

### Error Messages

- Add spans to functions, so a type error for a function can point to where the function is defined.
- Make it possible to return multiple spans per error. For example the type error message can point to the expression that has the wrong type, as well as the operator that expects a different one.
- Give spans a type, so operands and operators can be colored differently in error messages.
- Extend the spans of expressions that are the result of an evaluation, so they also point to the operator that did the evaluation (like `eval`, `each`). This requires support for discontinuous spans, or maybe expressions can just have different spans.

### REPL

Add a REPL. As far as debugging tools go, this is probably the most bang for the buck.

Could it even be possible to emulate some debugging features by implementing a `step` builtin, which evaluates the first word of a list, but leaves the rest of the list?

Over time, it might be possible to make the language so powerful, that a debugger can just be implemented as a library that is used with the REPL.
