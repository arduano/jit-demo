use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr,
};

use llvm_sys::{
    orc2::{LLVMOrcThreadSafeContextGetContext, LLVMOrcThreadSafeContextRef},
    prelude::LLVMModuleRef,
    target::{
        LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter,
        LLVM_InitializeNativeTarget,
    },
};
use shared::User;

use crate::JoinFilters;

use self::optimizing::Optimizer;

mod build_fn;
mod exec_engine;
mod io;
mod optimizing;

pub struct ModuleWithContext {
    pub module: LLVMModuleRef,
    pub orc_context: LLVMOrcThreadSafeContextRef,
}

pub(crate) fn to_c_str<'s>(mut s: &'s str) -> Cow<'s, CStr> {
    if s.is_empty() {
        s = "\0";
    }

    // Start from the end of the string as it's the most likely place to find a null byte
    if !s.chars().rev().any(|ch| ch == '\0') {
        return Cow::from(CString::new(s).expect("unreachable since null bytes are checked"));
    }

    unsafe { Cow::from(CStr::from_ptr(s.as_ptr() as *const _)) }
}

pub struct CallableJitFn {
    ee: exec_engine::JitExecutionEngine,
    fn_ptr: exec_engine::JitFunction,
}

impl CallableJitFn {
    pub unsafe fn execute(&self, vec: &[User]) -> Vec<User> {
        let mut output_vec = Vec::new();
        (self.fn_ptr)(vec, &mut output_vec);
        output_vec
    }
}

pub unsafe fn build_module(filters: &JoinFilters) -> CallableJitFn {
    LLVM_InitializeNativeTarget();
    LLVM_InitializeNativeAsmPrinter();
    LLVM_InitializeNativeAsmParser();

    let loaded = io::read_bytecode_module();

    let orc_context = loaded.orc_context;
    let module = loaded.module;
    let context = LLVMOrcThreadSafeContextGetContext(orc_context);

    let mut exec_engine = exec_engine::JitExecutionEngine::new();

    println!("Building module");
    let now = std::time::Instant::now();
    build_fn::build_fn("execute", loaded.module, context, filters);
    io::print_module_to_file(module, "jit.ll");
    Optimizer::new().optimize_module(module);
    io::print_module_to_file(module, "jit_opt.ll");
    dbg!(now.elapsed());

    println!("Adding module");
    let now = std::time::Instant::now();
    exec_engine.add_function("execute", loaded);
    dbg!(now.elapsed());

    CallableJitFn {
        fn_ptr: exec_engine.get_function_ptr("execute"),
        ee: exec_engine,
    }
}
