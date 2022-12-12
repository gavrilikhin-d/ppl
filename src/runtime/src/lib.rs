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


