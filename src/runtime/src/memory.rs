use libc::malloc;
use rug::Integer;

use crate::{integer_from_i64, integer_from_u64};

#[repr(C)]
pub struct MemoryAddress {
    pub value: *mut Integer,
}

/// # PPL
/// ```no_run
/// fn <address: MemoryAddress> as String -> String
/// ```
#[no_mangle]
pub extern "C" fn memory_address_as_string(address: MemoryAddress) -> *mut String {
    assert!(!address.value.is_null());
    let value = unsafe { &*address.value };

    let hex = format!("0x{}", value.to_string_radix(16).to_uppercase());
    let boxed = Box::new(hex);
    Box::into_raw(boxed)
}

/// # PPL
/// ```no_run
/// fn allocate <n: Integer> bytes -> MemoryAddress
/// ```
#[no_mangle]
pub extern "C" fn allocate_n_bytes(n: *const Integer) -> MemoryAddress {
    assert!(!n.is_null());
    let n = unsafe { &*n };

    let n = n.to_usize();
    if n.is_none() {
        return MemoryAddress {
            value: integer_from_i64(0),
        };
    }
    let n = n.unwrap();

    let address = unsafe { malloc(n) } as u64;

    MemoryAddress {
        value: integer_from_u64(address),
    }
}

/// # PPL
/// ```no_run
/// fn free <address: MemoryAddress>
/// ```
#[no_mangle]
pub extern "C" fn free_memory(address: MemoryAddress) {
    assert!(!address.value.is_null());
    let address = unsafe { &*address.value };

    let address = address.to_u64();
    if address.is_none() {
        return;
    }
    let address = address.unwrap();

    unsafe {
        libc::free(address as *mut libc::c_void);
    }
}
