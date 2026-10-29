//! `words!`: a family's part of the gate dispatch table.

/// The gates a module serves and the function that runs each, as a
/// `&[(Gate, Dispatch)]` that reva-words assembles into its dispatch
/// table.  `A | B => f;` maps gates to a handler
/// `fn(&mut Machine, Gate, &mut dyn Host) -> Result<Next, Fault>`;
/// `A => ret |m, h| expr;` is an adapter for a gate that only calls one
/// helper returning `Result<(), Fault>` and then returns to Forth.
///
/// ```ignore
/// pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
///     FClass | FClassNotail => f_class;
///     LBracket => ret |m, _h| m.lbracket();
/// };
/// ```
#[macro_export]
macro_rules! words {
    (@acc [$($out:expr,)*]) => { &[$($out,)*] };
    (@acc [$($out:expr,)*] $($g:ident)|+ => ret |$m:ident, $h:ident| $e:expr; $($rest:tt)*) => {
        $crate::words!(@acc [$($out,)* $((::reva_core::Gate::$g, $crate::words!(@ret $m $h $e)),)+] $($rest)*)
    };
    (@acc [$($out:expr,)*] $($g:ident)|+ => $f:path; $($rest:tt)*) => {
        $crate::words!(@acc [$($out,)* $((::reva_core::Gate::$g, $f as ::reva_core::Dispatch),)+] $($rest)*)
    };
    (@ret $m:ident $h:ident $e:expr) => {
        (|$m: &mut ::reva_core::Machine,
          _: ::reva_core::Gate,
          $h: &mut dyn ::reva_core::host::Host|
         -> ::core::result::Result<::reva_core::Next, ::reva_core::fault::Fault> {
            $e?;
            Ok(::reva_core::Next::Ret)
        }) as ::reva_core::Dispatch
    };
    ($($body:tt)*) => { $crate::words!(@acc [] $($body)*) };
}
