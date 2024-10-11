* Use `derive_builder`, `getset`, and `derive_more` whenever possible.
* Do not use positional attributes except for single field structs.
* Do not use public attributes on public structs. Use getters and setters instead.
* Do not use `get_`. Do use `set_` and `take_` when necessary. Use `_mut` for mutable getters.
* Functions that return bool should start with `is_` or `has_`.
* Always use `hasbrown` for `HashMap` and `HashSet`.
* It's better to use  .expect` than to use `if let Some/Ok` unless you have an else. This is because `if let` without an `else` could hide a `None` or `Err` without panicing.
