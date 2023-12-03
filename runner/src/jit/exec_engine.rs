use std::{
    alloc::Layout, collections::HashMap, ffi::CStr, mem, panic::PanicInfo, process::exit, ptr,
};

use llvm_sys::{
    error::LLVMGetErrorMessage,
    orc2::{lljit::*, *},
};
use shared::User;

use super::{to_c_str, ModuleWithContext};

pub type JitFunction = unsafe extern "C" fn(_vec: &[User], output_vec: *mut Vec<User>);

struct JitFunctionModule {
    resource_tracker: LLVMOrcResourceTrackerRef,
    function: JitFunction,
    orc_context: LLVMOrcThreadSafeContextRef,
}

pub struct JitExecutionEngine {
    orc_jit: LLVMOrcLLJITRef,
    main_jd: LLVMOrcJITDylibRef,
    functions: HashMap<String, JitFunctionModule>,
}

impl JitExecutionEngine {
    pub unsafe fn new() -> Self {
        let mut orc_jit = ptr::null_mut();
        let err = LLVMOrcCreateLLJIT(&mut orc_jit, ptr::null_mut());
        if !err.is_null() {
            panic!("Failed to create orc jit");
        }

        let main_jd = LLVMOrcLLJITGetMainJITDylib(orc_jit);

        let make_global_mapping = |name: &str, ptr: u64| {
            let exec_session = LLVMOrcLLJITGetExecutionSession(orc_jit);
            let intern = LLVMOrcExecutionSessionIntern(exec_session, to_c_str(name).as_ptr());

            let symbol = LLVMJITEvaluatedSymbol {
                Address: ptr as u64,
                Flags: LLVMJITSymbolFlags {
                    GenericFlags: LLVMJITSymbolGenericFlags::LLVMJITSymbolGenericFlagsCallable
                        as u8,
                    TargetFlags: LLVMJITSymbolGenericFlags::LLVMJITSymbolGenericFlagsCallable as u8,
                },
            };

            let pair = LLVMOrcCSymbolMapPair {
                Name: intern,
                Sym: symbol,
            };

            pair
        };

        // Orc JIT can pull functions from the current executable, so passing in all of these is optional, but still possible.
        // For example, you could use a custom allocator by overwriting the alloc functions.
        let mut all_mappings = vec![
            make_global_mapping("rust_begin_unwind", begin_unwind as u64),
            make_global_mapping("rust_eh_personality", rust_eh_personality as u64),
            // make_global_mapping("__rust_alloc", allocate as u64),
            // make_global_mapping("__rust_alloc_error_handler", alloc_error_handler as u64),
            // make_global_mapping("__rust_dealloc", deallocate as u64),
            // make_global_mapping("__rust_realloc", reallocate as u64),
            // make_global_mapping("__rust_alloc_zeroed", allocate_zeroed as u64),
            // make_global_mapping(
            //     "__rust_no_alloc_shim_is_unstable",
            //     &mut RUST_NO_ALLOC_SHIM as *mut _ as u64,
            // ),
        ];

        let materialization = LLVMOrcAbsoluteSymbols(all_mappings.as_mut_ptr(), all_mappings.len());

        LLVMOrcJITDylibDefine(main_jd, materialization);

        Self {
            orc_jit,
            main_jd,
            functions: HashMap::new(),
        }
    }

    pub unsafe fn add_function(&mut self, name: &str, mod_ctx: ModuleWithContext) {
        let orc_module = LLVMOrcCreateNewThreadSafeModule(mod_ctx.module, mod_ctx.orc_context);

        let resource_tracker = LLVMOrcJITDylibCreateResourceTracker(self.main_jd);
        let err = LLVMOrcLLJITAddLLVMIRModuleWithRT(self.orc_jit, resource_tracker, orc_module);
        if !err.is_null() {
            let err = CStr::from_ptr(LLVMGetErrorMessage(err)).to_string_lossy();
            panic!("Failed to add module: {}", err);
        }

        let mut compiled = 0;
        let err = LLVMOrcLLJITLookup(self.orc_jit, &mut compiled, to_c_str(&name).as_ptr());
        if !err.is_null() {
            let err = CStr::from_ptr(LLVMGetErrorMessage(err)).to_string_lossy();
            panic!("Failed to lookup function: {}", err);
        }

        let compiled = mem::transmute::<_, JitFunction>(compiled);

        self.functions.insert(
            name.to_string(),
            JitFunctionModule {
                resource_tracker,
                function: compiled,
                orc_context: mod_ctx.orc_context,
            },
        );
    }

    pub fn get_function_ptr(&self, name: &str) -> JitFunction {
        let function = self.functions.get(name).unwrap();
        function.function
    }

    pub unsafe fn remove_group_fn(&mut self, name: &str) {
        let function = self.functions.remove(name).unwrap();
        let err = LLVMOrcResourceTrackerRemove(function.resource_tracker);
        if !err.is_null() {
            let err = CStr::from_ptr(LLVMGetErrorMessage(err)).to_string_lossy();
            panic!("Failed to remove function: {}", err);
        }

        LLVMOrcReleaseResourceTracker(function.resource_tracker);
        LLVMOrcDisposeThreadSafeContext(function.orc_context);
    }
}

static mut RUST_NO_ALLOC_SHIM: u8 = 0;

unsafe extern "C" fn deallocate(ptr: *mut u8, size: usize, layout: usize) {
    println!("deallocate");
    std::alloc::dealloc(ptr, Layout::from_size_align_unchecked(size, layout));
}

unsafe extern "C" fn allocate(size: usize, layout: usize) -> *mut u8 {
    println!("allocate");
    std::alloc::alloc(Layout::from_size_align_unchecked(size, layout))
}

unsafe extern "C" fn allocate_zeroed(size: usize, layout: usize) -> *mut u8 {
    std::alloc::alloc_zeroed(Layout::from_size_align_unchecked(size, layout))
}

unsafe extern "C" fn reallocate(
    ptr: *mut u8,
    old_size: usize,
    align: usize,
    new_size: usize,
) -> *mut u8 {
    std::alloc::realloc(
        ptr,
        Layout::from_size_align_unchecked(old_size, align),
        new_size,
    )
}

unsafe extern "C" fn alloc_error_handler(_size: usize, _layout: usize) -> ! {
    println!("alloc_error_handler");
    panic!("alloc_error_handler")
}

extern "C" fn begin_unwind(info: &PanicInfo) {
    println!("begin_unwind");
    println!("{}", info);
    exit(1);
}

extern "C" fn rust_eh_personality() {
    println!("rust_eh_personality");
}
