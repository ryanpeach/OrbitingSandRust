Use `derive_builder` and `getset` whenever possible.

Do not use positional attributes except for single field structs.

Do not use `get_`. Do use `set_` and `take_` when necessary. Use `_mut` for mutable getters.

Functions that return bool should start with `is_` or `has_`.