# Darling

`darling` is a crate for proc macro authors, which enables parsing attributes into structs. It is heavily inspired by `serde` both in its internals and in its API.

# Example

```rust,ignore
#[macro_use]
extern crate darling;

#[derive(FromMetaItem)]
pub struct Lorem {
    #[darling(default = "local", rename = "sit")]
    ipsum: bool,
    #[darling(default)]
    dolor: Option<String>,
}
```

# Features
Darling's features are built to work well for real-world projects.

* **Defaults**: Supports struct- and field-level defaults, using the same path syntax as `serde`.
* **Field Renaming**: Fields can have different names in usage vs. the backing code.
* **Built-in conversions**: Darling provides native support for types in the standard library, as well as for types in the `syn` crate such as `Path` and `Ident`.

# To Do
[ ] Finish error-handling story
[ ] Add more type conversions
[ ] Improve error diagnostics