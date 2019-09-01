# Ideas

## Design

### Prelude

Since `std` is a module, it obvioulsy can't be the place where the module system is implemented. There should be an invisible prelude that does that, which is added to the front of the top-level module by the interpreter.

### Scopes

- Add a scope to each list. Calling `define` defines the function in the scope of the current list.
- Since each list must be defined in another list (except the top-level module, which is the implicit root list), lists, and therefore scopes, form a hierarchy. If a function is not found in a given scope, it should be searched for in the parent scope, and so on.
- It must be possible to export functions from a module. This could be done by explicitely pushing a list of list/symbol pairs to the stack at the end of a module. The list would then be `eval`ualed item by item when loading the list. This could all be implemented in Kari.

### Function Shadowing

At the moment, user-defined functions shadow extensions, which shadow builtins. It might be better to disallow shadowing altogether. Or maybe it's better to defer the decision until scopes are in the language.


## Implementation

### Builtin/Extension Input/Output

Builtins and extensions already need to define their input. It probably makes sense to make that directly available to them as an argument to the builtin/extension function. This would be less work and less error-prone. It might even be possible to take stack manipulation out of them completely, if outputs are also defined at some point.

### `Evaluator` as `Stream` of side effects

If I ever end up modelling side effects somehow, then `Evaluator` could be turned into a stream of side effects that the runner pulls out of it.

### Error Messages

- Add spans to functions, so a type error for a function can point to where the function is defined.
- Make it possible to return multiple spans per error. For example the type error message can point to the expression that has the wrong type, as well as the operator that expects a different one.
- Give spans a type, so operands and operators can be colored differently in error messages.
- Extend the spans of expressions that are the result of an evaluation, so they also point to the operator that did the evaluation (like `eval`, `each`). This requires support for discontinuous spans, or maybe expressions can just have different spans.
- When a function is not found, the error message should show the signatures of candidates with the same name, as well as the current stack.
- When a function definition conlicts with an existing definition, that definition should be shown.

### REPL

Add a REPL. As far as debugging tools go, this is probably the most bang for the buck.

Could it even be possible to emulate some debugging features by implementing a `step` builtin, which evaluates the first word of a list, but leaves the rest of the list?

Over time, it might be possible to make the language so powerful, that a debugger can just be implemented as a library that is used with the REPL.
