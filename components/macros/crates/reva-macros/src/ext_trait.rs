//! `ext_trait!`: methods of a type defined in another crate.

/// Methods of a type defined in another crate (Rust allows `impl Type`
/// only in the crate that defines it), written once with their bodies:
/// the macro declares the trait and implements it for the type, so
/// `self` is the value itself.  Attributes on a method (docs, `#[inline]`)
/// go to its implementation.
///
/// ```ignore
/// reva_macros::ext_trait! {
///     impl reva_x86_types::cpu::Cpu;
///     /// Flags.
///     pub trait Carry {
///         /// Set CF.
///         #[inline]
///         fn stc(&mut self) { self.cf = true; }
///     }
/// }
/// ```
#[macro_export]
macro_rules! ext_trait {
    (
        impl $ty:ty;
        $(#[$tm:meta])*
        pub trait $name:ident $(: $sup:path $(, $more:path)*)? {
            $( $(#[$m:meta])* fn $f:ident ( $($a:tt)* ) $(-> $r:ty)? $b:block )*
        }
    ) => {
        $(#[$tm])*
        pub trait $name $(: $sup $(+ $more)*)? {
            $( fn $f ( $($a)* ) $(-> $r)?; )*
        }
        impl $name for $ty {
            $( $(#[$m])* fn $f ( $($a)* ) $(-> $r)? $b )*
        }
    };
}
