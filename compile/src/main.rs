use std::borrow::Cow;
use std::ffi::CStr;
use std::ffi::CString;

use std::path::Path;
use std::path::PathBuf;
use std::ptr;

use llvm_sys::bit_reader::*;
use llvm_sys::bit_writer::*;
use llvm_sys::core::*;

use llvm_sys::linker::*;

use llvm_sys::target::LLVM_InitializeNativeTarget;
use llvm_sys::target_machine::*;

use llvm_sys::transforms::pass_builder::*;

use llvm_sys::LLVMLinkage;
use llvm_sys::LLVMModule;
use toml::Value;

fn execute_shell_command(command: &str) {
    use std::process::Command;
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("failed to execute process")
        .wait()
        .expect("failed to wait for process");
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

fn find_package_name<P: AsRef<Path>>(path: P) -> String {
    let cargo_toml = path.as_ref().join("Cargo.toml");

    let config = std::fs::read_to_string(cargo_toml).unwrap();
    let data: Value = toml::from_str(&config).unwrap();

    let name_field = data
        .get("package")
        .and_then(|x| x.get("name"))
        .and_then(Value::as_str);

    name_field.unwrap().to_string()
}

#[derive(Debug)]
struct ProjectLlvmBc {
    project: PathBuf,
    deps: Vec<PathBuf>,
}

fn recompile_project_into_llvm_bc(project_path: impl AsRef<Path>) -> ProjectLlvmBc {
    let project_path = project_path.as_ref().canonicalize().unwrap();
    let package_name = find_package_name(&project_path);

    let target_path = project_path.join("target").join("x86_64-unknown-linux-gnu");
    // Delete
    std::fs::remove_dir_all(&target_path).unwrap_or_default();

    // Compile
    let absolute_path = project_path.canonicalize().unwrap();
    let absolute_path_str = absolute_path.to_str().unwrap();
    execute_shell_command(&format!("cd {absolute_path_str} && RUSTFLAGS=\"--emit=llvm-bc\" cargo build --package=functions --release -Z build-std=\"core,alloc\" --target x86_64-unknown-linux-gnu --target-dir ./target"));

    // Find all compiled deps
    let deps_path = target_path.join("release").join("deps");
    let mut deps = Vec::new();

    dbg!(&deps_path);

    for entry in std::fs::read_dir(deps_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let path_str = path.to_str().unwrap();
        if path_str.ends_with(".bc") {
            deps.push(path);
        }
    }

    let dep_name_regex = regex::Regex::new("(.+)\\-.{16}\\.bc").unwrap();
    let index_of_package = deps
        .iter()
        .position(|x| {
            let filename = x.file_name().unwrap().to_str().unwrap();

            let captures = dep_name_regex.captures(filename).unwrap();
            let name = captures.get(1).unwrap().as_str();

            name == package_name
        })
        .unwrap();

    let project = deps.remove(index_of_package);

    ProjectLlvmBc { project, deps }
}

unsafe fn mark_all_module_items_for_linking(module: *mut LLVMModule) {
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

unsafe fn replace_linked_with_private(module: *mut LLVMModule) {
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

unsafe fn mark_all_as_private(module: *mut LLVMModule) {
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

unsafe fn module_purge_module_asm(module: *mut LLVMModule) {
    LLVMSetModuleInlineAsm2(module, to_c_str("").as_ptr(), 0);
}

fn link_llvm_bincode(
    files: &ProjectLlvmBc,
    output: impl AsRef<Path>,
    output_ll: Option<impl AsRef<Path>>,
) {
    unsafe {
        // Set up a context, module and builder in that context.
        let context = LLVMContextCreate();

        let read_module = |path: &Path| {
            let mut module = ptr::null_mut();

            let file = std::fs::read(path).unwrap();
            let mod_name = to_c_str("module");
            let buffer = LLVMCreateMemoryBufferWithMemoryRangeCopy(
                file.as_ptr() as *const libc::c_char,
                file.len(),
                mod_name.as_ptr(),
            );

            let code = LLVMParseBitcodeInContext2(context, buffer, &mut module);
            if code != 0 {
                println!("code, {code}");
                panic!("failed to load module");
            }

            module
        };

        let project_module = read_module(&files.project);

        for _ in 0..2 {
            // Do it twice, first time to make sure all the declarations are added, second time to make sure
            // all the declarations have been populated. I'm not sure if there's a more efficient way of
            // doing this while making sure all external items stay private.

            for file in &files.deps {
                let module = read_module(&file);
                module_purge_module_asm(module);
                mark_all_module_items_for_linking(module);
                let code = LLVMLinkModules2(project_module, module);
                if code != 0 {
                    panic!("failed to link modules");
                }
            }
        }

        replace_linked_with_private(project_module);

        let pass_builder_opts = LLVMCreatePassBuilderOptions();
        let triple = LLVMGetTarget(project_module);

        LLVM_InitializeNativeTarget();

        let target = LLVMGetTargetFromName(to_c_str("x86-64").as_ptr());

        let machine_target = LLVMCreateTargetMachine(
            target,
            triple,
            to_c_str("generic").as_ptr(),
            to_c_str("").as_ptr(),
            LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );

        LLVMRunPasses(
            project_module,
            to_c_str("default<O3>").as_ptr(),
            machine_target,
            pass_builder_opts,
        );

        mark_all_as_private(project_module);

        let output_path = output.as_ref();
        let output_path = output_path.to_str().unwrap();
        LLVMWriteBitcodeToFile(project_module, to_c_str(output_path).as_ptr());

        if let Some(output_ll) = output_ll {
            let output_path = output_ll.as_ref();
            let output_path = output_path.to_str().unwrap();
            let err_msg = ptr::null_mut();
            let error = LLVMPrintModuleToFile(
                project_module,
                to_c_str(output_path).as_ptr(),
                err_msg as *mut *mut libc::c_char,
            );

            if error != 0 {
                panic!(
                    "failed to write module to file, {}",
                    CStr::from_ptr(err_msg).to_str().unwrap()
                );
            }
        }
    }
}

fn main() {
    let bincode = recompile_project_into_llvm_bc("./functions");
    dbg!(&bincode);
    link_llvm_bincode(
        &bincode,
        "./functions/compiled.bc",
        Some("./functions/compiled.ll"),
    );
}
