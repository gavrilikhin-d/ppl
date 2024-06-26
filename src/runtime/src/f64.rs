use crate::{Rational, String};

type F64 = f64;

/// Converts `F64` to `String`
///
/// Runtime for builtin ppl's function:
/// # PPL
/// ```no_run
/// /// Convert `F64` to `String`
/// @mangle_as("f64_as_string")
/// fn String from <:F64> -> String
/// ```
#[no_mangle]
pub extern "C" fn f64_as_string(d: F64) -> String {
    d.to_string().into()
}

/// Negates f64
///
/// # PPL
/// ```no_run
/// @mangle_as("minus_f64")
/// fn - <:F64> -> F64
/// ```
#[no_mangle]
pub extern "C" fn minus_f64(d: F64) -> F64 {
    -d
}

/// Add 2 f64s
///
/// # PPL
/// ```no_run
/// @mangle_as("f64_plus_f64")
/// fn <:F64> + <:F64> -> F64
/// ```
#[no_mangle]
pub extern "C" fn f64_plus_f64(x: F64, y: F64) -> F64 {
    x + y
}

/// Multiply 2 F64s
///
/// # PPL
/// ```no_run
/// @mangle_as("f64_star_f64")
/// fn <:F64> * <:F64> -> F64
/// ```
#[no_mangle]
pub extern "C" fn f64_star_f64(x: F64, y: F64) -> F64 {
    x * y
}

/// Create f64 from rational
///
/// # PPL
/// ```no_run
/// /// Convert `Rational` to `F64`
/// @mangle_as("f64_from_rational")
/// fn F64 from <:Rational> -> F64
/// ```
#[no_mangle]
pub extern "C" fn f64_from_rational(r: Rational) -> F64 {
    r.as_ref().to_f64()
}

/// Create rational from f64
///
/// # PPL
/// ```no_run
/// /// Convert `Rational` to `F64`
/// @mangle_as("rational_from_f64")
/// fn Rational from <:F64> -> Rational
/// ```
#[no_mangle]
pub extern "C" fn rational_from_f64(d: F64) -> Rational {
    rug::Rational::from_f64(d).unwrap().into()
}
