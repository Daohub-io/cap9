#![no_std]

#[no_mangle]
pub extern fn extcodesize( _address: *const u8) -> i32 {
    panic!("extcodesize not available")
}

#[no_mangle]
pub extern fn extcodecopy( _dest: *mut u8, _address: *const u8) {
    panic!("extcodecopy not available");
}

// #[no_mangle]
// pub extern fn dcall(
//             gas: i64,
//             address: *const u8,
//             input_ptr: *const u8,
//             input_len: u32,
//             result_ptr: *mut u8,
//             result_len: u32,
//     ) -> i32 {
//         panic!("dcall not available")
//     }

#[no_mangle]
pub extern fn cap9_syscall_low(_input_ptr: *const u8, _input_len: u32, _result_ptr: *mut u8, _result_len: u32) -> i32 {
    panic!("cap9_syscall_low not available")
}

#[no_mangle]
pub extern fn gasleft() -> i64 {
    panic!("gasleft not available");
}

#[no_mangle]
pub extern fn call_code() -> i64 {
    panic!("call_code not available");
}


#[no_mangle]
pub extern fn result_length() -> i64 {
    panic!("result_length not available");
}


#[no_mangle]
pub extern fn fetch_result() -> i64 {
    panic!("fetch_result not available");
}
