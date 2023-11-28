#![no_std]
#![allow(improper_ctypes_definitions)]

extern crate alloc;

use alloc::vec::Vec;
use shared::User;

// ======
// Misc
// ======

#[no_mangle]
#[inline(always)]
// An example of rust ABI doing weird things. Keep an eye out.
pub unsafe extern "C" fn fn_sig_bad(_vec: &[User]) -> Vec<User> {
    // Function used purely for copying the function signature in LLVM
    unimplemented!()
}

#[no_mangle]
#[inline(always)]
pub unsafe extern "C" fn fn_sig(_vec: &[User], _output_vec: *mut Vec<User>) {
    // Function used purely for copying the function signature in LLVM
    unimplemented!()
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn filter_fn_sig(_user: &User) -> bool {
    // Function used purely for copying the function signature in LLVM
    unimplemented!()
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn test_return_str() -> &'static str {
    // Example of how a string compiles under the hood
    "hello world"
}

#[repr(C)]
pub struct SeparatedStr {
    pub ptr: *const u8,
    pub len: u64,
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn separated_str_as_str(separated: *mut SeparatedStr) -> &'static str {
    unsafe {
        let separated = &*separated;
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            separated.ptr,
            separated.len as usize,
        ))
    }
}

// ======
// Processing
// ======

#[no_mangle]
#[inline(always)]
pub extern "C" fn run_filter(
    vec: &[User],
    output_vec: *mut Vec<User>, // Expects null
    filter: extern "C" fn(&User) -> bool,
) {
    unsafe {
        *output_vec = Vec::new();
        (*output_vec).extend(vec.iter().filter(|user| filter(user)).cloned());
    }
}

// ======
// Filters
// ======

#[no_mangle]
#[inline(always)]
pub extern "C" fn filter_str_contains(s: &str, substr: &str) -> bool {
    s.contains(substr)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn filter_str_equals(s: &str, other: &str) -> bool {
    s == other
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn filter_str_starts_with(s: &str, prefix: &str) -> bool {
    s.starts_with(prefix)
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn filter_str_ends_with(s: &str, suffix: &str) -> bool {
    s.ends_with(suffix)
}

// #[no_mangle]
// #[inline(always)]
// pub extern "C" fn filter_str_matches(s: &str, regex: &str) -> bool {
//     regex::Regex::new(regex).unwrap().is_match(s)
// }

// ======
// User
// ======

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_email(user: &User) -> &str {
    &user.email
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_gender(user: &User) -> &str {
    &user.gender
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_phone_number(user: &User) -> &str {
    &user.phone_number
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_location_street(user: &User) -> &str {
    &user.location.street
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_location_city(user: &User) -> &str {
    &user.location.city
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_location_state(user: &User) -> &str {
    &user.location.state
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_username(user: &User) -> &str {
    &user.username
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_password(user: &User) -> &str {
    &user.password
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_first_name(user: &User) -> &str {
    &user.first_name
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_last_name(user: &User) -> &str {
    &user.last_name
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_title(user: &User) -> &str {
    &user.title
}

#[no_mangle]
#[inline(always)]
pub extern "C" fn user_get_field_picture(user: &User) -> &str {
    &user.picture
}
