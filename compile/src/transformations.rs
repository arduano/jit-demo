use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;

use std::path::Path;
use std::path::PathBuf;
use std::ptr;

use llvm_sys::bit_reader::*;
use llvm_sys::bit_writer::*;
use llvm_sys::core::*;
use llvm_sys::LLVMLinkage;

use llvm_sys::linker::*;

use llvm_sys::prelude::*;
use llvm_sys::target::LLVM_InitializeNativeTarget;
use llvm_sys::target_machine::*;

use llvm_sys::transforms::pass_builder::*;

use toml::Value;

use crate::to_c_str;

pub unsafe fn mark_all_module_items_for_linking(module: LLVMModuleRef) {
    let mut f = LLVMGetFirstGlobal(module);
    while !f.is_null() {
        LLVMSetLinkage(f, LLVMLinkage::LLVMLinkOnceAnyLinkage);
        f = LLVMGetNextGlobal(f);
    }

    let mut f = LLVMGetFirstGlobalAlias(module);
    while !f.is_null() {
        LLVMSetLinkage(f, LLVMLinkage::LLVMLinkOnceAnyLinkage);
        f = LLVMGetNextGlobalAlias(f);
    }

    let mut f = LLVMGetFirstGlobalIFunc(module);
    while !f.is_null() {
        LLVMSetLinkage(f, LLVMLinkage::LLVMLinkOnceAnyLinkage);
        f = LLVMGetNextGlobalIFunc(f);
    }

    let mut f = LLVMGetFirstFunction(module);
    while !f.is_null() {
        LLVMSetLinkage(f, LLVMLinkage::LLVMLinkOnceAnyLinkage);
        f = LLVMGetNextFunction(f);
    }
}

pub unsafe fn replace_linked_with_private(module: LLVMModuleRef) {
    let mut f = LLVMGetFirstGlobal(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current == LLVMLinkage::LLVMLinkOnceAnyLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobal(f);
    }

    let mut f = LLVMGetFirstGlobalAlias(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current == LLVMLinkage::LLVMLinkOnceAnyLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobalAlias(f);
    }

    let mut f = LLVMGetFirstGlobalIFunc(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current == LLVMLinkage::LLVMLinkOnceAnyLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobalIFunc(f);
    }

    let mut f = LLVMGetFirstFunction(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current == LLVMLinkage::LLVMLinkOnceAnyLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextFunction(f);
    }
}

pub unsafe fn mark_all_as_private(module: LLVMModuleRef) {
    let mut f = LLVMGetFirstGlobal(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current != LLVMLinkage::LLVMExternalLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobal(f);
    }

    let mut f = LLVMGetFirstGlobalAlias(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current != LLVMLinkage::LLVMExternalLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobalAlias(f);
    }

    let mut f = LLVMGetFirstGlobalIFunc(module);
    while !f.is_null() {
        let current = LLVMGetLinkage(f);
        if current != LLVMLinkage::LLVMExternalLinkage {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextGlobalIFunc(f);
    }

    let mut f = LLVMGetFirstFunction(module);
    while !f.is_null() {
        if LLVMIsDeclaration(f) == 0 {
            LLVMSetLinkage(f, LLVMLinkage::LLVMPrivateLinkage);
        }

        f = LLVMGetNextFunction(f);
    }
}

pub unsafe fn module_purge_module_asm(module: LLVMModuleRef) {
    LLVMSetModuleInlineAsm2(module, to_c_str("").as_ptr(), 0);
}
