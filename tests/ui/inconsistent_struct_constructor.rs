//@aux-build:proc_macros.rs

#![warn(clippy::inconsistent_struct_constructor)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::unnecessary_operation)]
#![allow(clippy::no_effect)]
#![allow(dead_code)]

extern crate proc_macros;

#[derive(Default)]
struct Foo {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Default)]
#[allow(clippy::inconsistent_struct_constructor)]
struct Bar {
    x: i32,
    y: i32,
    z: i32,
}

mod without_base {
    use super::Foo;

    #[proc_macros::inline_macros]
    fn test() {
        let x = 1;
        let y = 1;
        let z = 1;

        // Should lint.
        Foo { y, x, z };
        //~^ inconsistent_struct_constructor

        // Should NOT lint.
        // issue #7069.
        inline!({
            let x = 1;
            let y = 1;
            let z = 1;
            Foo { y, x, z }
        });

        // Should NOT lint because the order is the same as in the definition.
        Foo { x, y, z };

        // Should NOT lint because z is not a shorthand init.
        Foo { y, x, z: z };
    }
}

mod with_base {
    use super::Foo;

    fn test() {
        let x = 1;
        let z = 1;

        // Should lint.
        Foo {
            z,
            x,
            ..Default::default()
        };
        //~^^^^ inconsistent_struct_constructor

        // Should NOT lint because the order is consistent with the definition.
        Foo {
            x,
            z,
            ..Default::default()
        };

        // Should NOT lint because z is not a shorthand init.
        Foo {
            z: z,
            x,
            ..Default::default()
        };
    }
}

mod with_allow_ty_def {
    use super::Bar;

    fn test() {
        let x = 1;
        let y = 1;
        let z = 1;

        // Should NOT lint because `Bar` is defined with `#[allow(clippy::inconsistent_struct_constructor)]`
        Bar { y, x, z };
    }
}

fn main() {}
