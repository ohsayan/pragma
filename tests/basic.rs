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

#[test]
fn try_() { /* just ensure it compiles */ }
