# Bugs

## Errors look weird

Many of them don't have any spans any more, except those from the call stack. This leads to the information being presented in a weird way, even though all the information is there.

## Spans from multiple files are merged regularly

The code that merges spans is obviously made with the intent of supporting merges between spans in the same file, but it is regularly called with spans from different files. The effect of this are likely to be incorrect error messages for code involving such spans.

I don't know what's supposed to happen here. Maybe it should just be possible to merge spans from different files. Maybe a span should just be a continuous span in one file, and merging spans from different files should result in a different type. Maybe discontinuous spans in the same file need to be handled too somehow.

It might make sense to turn spans into an enum that has variants for each case.

Additional thought: To know whether merging two spans will result in a continuous span, spans also need to keep track of their neighbouring whitespace. Otherwise, there's no way to find out whether two spans are neighbours, or if there's something in between.
