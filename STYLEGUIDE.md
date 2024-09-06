# Getters, Setters

## Builders

Builders should not use `set_` methods, opting just for the literal name of the parameter.

## Everything else

Should use a `get_`, `set_`, `take_`, or `calc_` prefix.

## Takes

Takes should run on `Option` types, and should return an Option. They should be acompanied by a `set_` method, to put it back. It's better to not use these if you don't have to. Borrow, copy, or clone instead.

## Why do we use getters and setters

Because its hard to predict the future. If you need to change the way a value is stored, you can do so without changing the interface. This is a big project going forward, so I don't want to take any chances.

## Documentation References

Use references in documentation to avoid copying documentation. For example, `set_name` should just say "Set [name]" to reference the `name` field and its documentation.