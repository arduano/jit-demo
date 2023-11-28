use std::ptr;

use llvm_sys::{
    orc2::{
        LLVMOrcDisposeJITTargetMachineBuilder, LLVMOrcJITTargetMachineBuilderDetectHost,
        LLVMOrcJITTargetMachineBuilderGetTargetTriple,
    },
    prelude::LLVMModuleRef,
    target_machine::*,
    transforms::pass_builder::*,
};

use super::to_c_str;

pub struct Optimizer {
    target_machine: LLVMTargetMachineRef,
    pass_builder_opts: LLVMPassBuilderOptionsRef,
}

impl Optimizer {
    pub fn new() -> Self {
        unsafe {
            let cpu = LLVMGetHostCPUName();
            if cpu.is_null() {
                panic!("failed to get cpu");
            }

            let target = LLVMGetTargetFromName(to_c_str("x86-64").as_ptr());
            if target.is_null() {
                panic!("failed to get target");
            }

            let mut jit_builder = ptr::null_mut();
            let err = LLVMOrcJITTargetMachineBuilderDetectHost(&mut jit_builder);
            if !err.is_null() {
                panic!("failed to create jit builder");
            }

            let triple = LLVMOrcJITTargetMachineBuilderGetTargetTriple(jit_builder);
            if triple.is_null() {
                panic!("failed to get triple");
            }

            LLVMOrcDisposeJITTargetMachineBuilder(jit_builder);

            let target_machine = LLVMCreateTargetMachine(
                target,
                triple,
                cpu,
                to_c_str("").as_ptr(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelJITDefault,
            );

            Self {
                target_machine,
                pass_builder_opts: LLVMCreatePassBuilderOptions(),
            }
        }
    }

    pub unsafe fn optimize_module(&self, module: LLVMModuleRef) {
        LLVMRunPasses(
            module,
            to_c_str("default<O3>").as_ptr(),
            self.target_machine,
            self.pass_builder_opts,
        );
    }
}

impl std::ops::Drop for Optimizer {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeTargetMachine(self.target_machine);
            LLVMDisposePassBuilderOptions(self.pass_builder_opts);
        }
    }
}
