
use rug::Integer;

/// Runtime for PPL's builtin functions
#[repr(C)]
pub struct None {
	_data: [u8; 0],
    _marker:
	core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Default constructor for PPL's [`None`](ppl::semantics::Type::None) type
#[no_mangle]
pub extern "C" fn none() -> *const None {
	core::ptr::null::<None>()
}

/// Prints none value
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn print <x: None> -> None
/// ```
#[no_mangle]
pub extern "C" fn print_none(none: *const None) -> *const None {
	println!("none");
	none
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_i64(value: i64) -> *mut Integer {
	let boxed = Box::new(value.into());
	Box::into_raw(boxed)
}

/// Construct [`Integer`](ppl::semantics::Type::Integer) from a C string
#[no_mangle]
pub extern "C" fn integer_from_c_string(str: *const i8) -> *mut Integer {
	let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
	let str = c_str.to_str().unwrap();
	let boxed = Box::new(str.parse::<Integer>().unwrap());
	Box::into_raw(boxed)
}

/// Construct [`String`](ppl::semantics::Type::String) from a C string
/// and length
#[no_mangle]
pub extern "C" fn string_from_c_string_and_length(str: *const i8, _len: u64) -> *mut String {
	let c_str = unsafe { core::ffi::CStr::from_ptr(str) };
	let str = c_str.to_str().unwrap();
	let boxed = Box::new(str.to_string());
	Box::into_raw(boxed)
}

/// Prints none value
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn print <str: String> -> None
/// ```
#[no_mangle]
pub extern "C" fn print_string(str: *const String) -> *const None {
	if str.is_null() {
		panic!("null pointer passed to print_string");
	}
	println!("{}", unsafe { &*str });
	none()
}

/// Converts `Integer` to `String`
///
/// Runtime for builtin ppl's function:
/// ```ppl
/// fn <:Integer> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn integer_as_string(i: *const Integer) -> *mut String {
	if i.is_null() {
		panic!("null pointer passed to integer_as_string");
	}
	let boxed = Box::new(unsafe { &*i }.to_string());
	Box::into_raw(boxed)
}

// IMPORTANT: don't forget to update global mapping after adding new function!!!