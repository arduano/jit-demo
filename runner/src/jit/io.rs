use std::{ffi::CStr, process::exit, ptr};

use llvm_sys::{
    bit_reader::LLVMParseBitcodeInContext2,
    core::{LLVMCreateMemoryBufferWithMemoryRangeCopy, LLVMPrintModuleToFile},
    orc2::{
        LLVMOrcCreateNewThreadSafeContext, LLVMOrcThreadSafeContextGetContext,
        LLVMOrcThreadSafeContextRef,
    },
    prelude::{LLVMContextRef, LLVMModuleRef},
};
use serde_json::de::Read;

use super::{to_c_str, ModuleWithContext};

/// Read the compiled bytecode into a module (in a new Orc context)
pub unsafe fn read_bytecode_module() -> ModuleWithContext {
    let file = include_bytes!("../../../functions/compiled.bc");
    let mod_name = to_c_str("module");
    let buffer = LLVMCreateMemoryBufferWithMemoryRangeCopy(
        file.as_ptr() as *const libc::c_char,
        file.len(),
        mod_name.as_ptr(),
    );

    let orc_context = LLVMOrcCreateNewThreadSafeContext();
    let context = LLVMOrcThreadSafeContextGetContext(orc_context);

    let mut module = ptr::null_mut();

    let code = LLVMParseBitcodeInContext2(context, buffer, &mut module);
    if code != 0 {
        panic!("code, {code}");
    }

    ModuleWithContext {
        module,
        orc_context,
    }
}

/// Print a module to a file for debug reasons
pub unsafe fn print_module_to_file(module: LLVMModuleRef, filename: &str) {
    let err = ptr::null_mut();
    LLVMPrintModuleToFile(module, to_c_str(filename).as_ptr(), err);
    if !err.is_null() {
        let err = CStr::from_ptr(err as *const _);
        println!("err, {:?}", err);
        exit(1);
    }
}
