# `pragma` Macro

Mostly an experimentation macro. The `pragma!` macro provides a powerful experimental inline DSL for conditional item inclusion in Rust code. It lets you specify conditions on functions, modules, or other items and automatically generates `#[cfg(...)]` attributes with supporting logic (including `and`, `or`, `not`, and parenthesized groups) for complex conditional compilation scenarios.

## Full example

Here's a full example that utilizes all of `pragma`'s language extensions:

```rust
use pragma::pragma;

pragma! {
    /// this function is public if `target_pointer_width = "64"`, otherwise private
    pub (if target_pointer_width = "64") fn wide_pointer_fn() {
        println!("64-bit pointer width!");
    }

    /// a module that is included if `test` is enabled, private otherwise
    pub (if test) mod test_mod {
        pub fn inside() { println!("Testing mode!"); }
    }

    /// a function that only appears if `target_pointer_width = "32"`
    (if target_pointer_width = "32") fn narrow_pointer_fn() {}

    /// a more complex condition that checks multiple cfg attributes
    (if target_pointer_width = "64" and (target_pointer_width = "16" or not(debug_assertions))) fn fancy_fn() {
        println!("Fancy conditional logic!");
    }

    /// an unconditional item
    static _UNCONDITIONAL: &'static str = "Always here!";
}
```

## Language extensions

1. **Simple Conditional Items**:
   You can write:

   ```rust
   pragma! {
       (if test) fn do_something() {
           println!("Running with `test` feature!");
       }
   }
   ```

2. **Visibility-Aware Items**:
   If you specify a visibility modifier before `(if condition)`, two versions of the item are generated:

   - A public version with `#[cfg(condition)]`
   - A private version with `#[cfg(not(condition))]`

   For example:

   ```rust
   pragma! {
       /// this function is public if `test` is enabled, otherwise a private fallback is included
       pub (if test) fn conditional_public() {}
   }
   ```

   This expands to:

   ```rust
   #[cfg(test)]
   pub fn conditional_public() {}
   
   #[cfg(not(test))]
   fn conditional_public() {}
   ```

   Without any visibility specified, only a single `#[cfg(condition)]` version of the item is generated:

   ```rust
   pragma! {
       (if test) fn single_version() {}
   }
   ```

   Expands to:

   ```rust
   #[cfg(test)]
   fn single_version() {}
   ```

3. **Unconditional Items**:
   You can mix normal, unconditional items with conditional ones. For example:

   ```rust
   pragma! {
       (if test) fn conditional_fn() {}
       static GLOBAL_VAR: i32 = 42; // no conditions here, always included
   }
   ```

4. **Modules With Conditions**:
   You can also apply conditions to entire modules:

   ```rust
   pragma! {
       pub (if test) mod conditional_mod {
           fn inside() {}
       }
   }
   ```

   This expands to a public module if `test` is enabled, or a private module otherwise.

5. **Complex Conditions**:
   The conditions inside `(if ...)` support a custom DSL with `and`, `or`, `not`, parentheses for grouping, and key-value checks:

   - Use `and` and `or` for logical conjunction and disjunction.
   - Wrap conditions in parentheses to control evaluation.
   - Use `not(...)` for negation.
   - Use `key = "value"` for cfg key-value pairs, and bare `key` for boolean cfg options.

   Examples:

   ```rust
   pragma! {
       // matches if target_pointer_width = "64" or target_pointer_width = "16"
       (if target_pointer_width = "64" or target_pointer_width = "16") fn special_fn() {}
   
       // matches if target_pointer_width = "64" and (target_pointer_width = "16" or not(debug_assertions))
       (if target_pointer_width = "64" and (target_pointer_width = "16" or not(debug_assertions))) fn complex_fn() {}
   
       // you can also negate conditions
       (if not(test)) fn not_test_fn() {}
   }
   ```

   Expands to:

   ```rust
   #[cfg(all(target_pointer_width = "64", target_pointer_width = "16"))]
   fn special_fn() {}
   
   #[cfg(all(
       target_pointer_width = "64",
       any(target_pointer_width = "16", not(debug_assertions))
   ))]
   fn complex_fn() {}
   
   #[cfg(not(test))]
   fn not_test_fn() {}
   ```

## Motivation

If you're wondering why this was written in the first place, then the answer is:
(1) having used C I have been very used to pre-processor directives and while the same can often be done with `#[cfg(..)]` I feel "more at home" using something more C-style. Of course, ideally I would be able to apply it to a whole module file without explicit invocation, but from my understanding of Rust macros, that isn't currently possible.
(2) experimenting to test the limits of what can and cannot be done with procedural macros

I published this here if someone finds it useful.

Say you have something like this in C:

```c
#ifdef FEATURE_FLAG
// struct definition when FEATURE_FLAG is enabled
struct MyStruct {
    int a;
    double b;
};
#else
// struct definition when FEATURE_FLAG is not enabled
struct MyStruct {
    char x;
    float y;
};
#endif
```

Then you can write something similar in Rust using this macro:

```rust
pragma! {
    (if feature = FEATURE_FLAG) struct MyStruct {
        a: i32,
        b: f64,
    }
    (if not(feature = FEATURE_FLAG)) struct MyStruct {
        x: i8,
        y: f32
    }
}
```

In vanilla Rust, this would be:

```rust
#[cfg(feature = "FEATURE_FLAG")]
struct MyStruct { .. }
#[cfg(not(feature = "FEATURE_FLAG"))]
struct MyStruct { .. }
```

## License

This crate is distributed under the [Apache-2.0 License](./LICENSE).
