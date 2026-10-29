//! `machine_trait!`: methods of reva-core's `Machine` defined in other crates.

/// Methods of reva-core's `Machine` defined outside this crate: Rust allows `impl
/// Machine` only here, so reva-words and reva-vlib write theirs as an
/// extension trait.  The methods are written once, with their bodies; the
/// macro declares the trait and implements it for `Machine`, so `self` is
/// the machine itself (fields, its own methods, two-phase borrows).
///
/// ```ignore
/// reva_macros::machine_trait! {
///     /// The virtual library `example`.
///     pub trait Example: OtherTrait {
///         /// Answer 42.
///         fn answer(&mut self) -> u32 { 42 }
///     }
/// }
/// ```
#[macro_export]
macro_rules! machine_trait {
    (
        $(#[$tm:meta])*
        pub trait $name:ident $(: $sup:path $(, $more:path)*)? {
            $( $(#[$m:meta])* fn $f:ident ( $($a:tt)* ) $(-> $r:ty)? $b:block )*
        }
    ) => {
        $(#[$tm])*
        pub trait $name $(: $sup $(+ $more)*)? {
            $( $(#[$m])* fn $f ( $($a)* ) $(-> $r)?; )*
        }
        impl $name for ::reva_core::Machine {
            $( $(#[$m])* fn $f ( $($a)* ) $(-> $r)? $b )*
        }
    };
}
