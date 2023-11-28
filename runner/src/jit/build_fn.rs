use llvm_sys::core::*;
use llvm_sys::prelude::{
    LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMValueRef,
};
use llvm_sys::*;

use crate::{Field, Filter, JoinFilters};

use super::to_c_str;

struct FnBuilder {
    module: LLVMModuleRef,
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    user_arg: LLVMValueRef,
    str_counter: usize,
}

pub unsafe fn build_filter_fn(
    name: &str,
    module: LLVMModuleRef,
    context: LLVMContextRef,
    filters: &JoinFilters,
) {
    // Grab the function signature we want to copy and add it to the module
    let fn_val = LLVMGetNamedFunction(module, to_c_str("filter_fn_sig").as_ptr());
    let fn_type = LLVMGlobalGetValueType(fn_val);
    let fn_value = LLVMAddFunction(module, to_c_str(name).as_ptr(), fn_type);

    // Function should be private
    LLVMSetLinkage(fn_value, LLVMLinkage::LLVMPrivateLinkage);

    let entry_block = LLVMAppendBasicBlockInContext(context, fn_value, to_c_str("entry").as_ptr());
    let user_arg = LLVMGetParam(fn_value, 0);

    let fail_block = LLVMAppendBasicBlockInContext(context, fn_value, to_c_str("fail").as_ptr());
    let success_block =
        LLVMAppendBasicBlockInContext(context, fn_value, to_c_str("success").as_ptr());

    let builder = LLVMCreateBuilderInContext(context);

    // Build fail condition (return false)
    LLVMPositionBuilderAtEnd(builder, fail_block);
    let false_value = LLVMConstInt(LLVMInt1TypeInContext(context), 0, 0);
    LLVMBuildRet(builder, false_value);

    // Build success condition (return true)
    LLVMPositionBuilderAtEnd(builder, success_block);
    let true_value = LLVMConstInt(LLVMInt1TypeInContext(context), 1, 0);
    LLVMBuildRet(builder, true_value);

    // Position at entry block
    LLVMPositionBuilderAtEnd(builder, entry_block);

    let mut builder = FnBuilder {
        module,
        context,
        builder,
        user_arg,
        str_counter: 0,
    };

    builder.build_join_filter(filters, fail_block, success_block);

    LLVMDisposeBuilder(builder.builder);
}

impl FnBuilder {
    unsafe fn make_call(
        &self,
        fn_name: &str,
        result_name: &str,
        args: &mut [*mut LLVMValue],
    ) -> LLVMValueRef {
        let f = LLVMGetNamedFunction(self.module, to_c_str(fn_name).as_ptr());

        if f.is_null() {
            panic!("Function {} not found", fn_name);
        }

        let ty = LLVMGlobalGetValueType(f);

        LLVMBuildCall2(
            self.builder,
            ty,
            f,
            args.as_mut_ptr(),
            args.len() as u32,
            to_c_str(result_name).as_ptr(),
        )
    }

    unsafe fn build_global_str(&mut self, text: &str) -> *mut LLVMValue {
        let global_name = format!("str_{}", self.str_counter);
        self.str_counter += 1;

        let str_characters_ty =
            LLVMArrayType2(LLVMInt8TypeInContext(self.context), text.len() as u64);
        let len_ty = LLVMIntTypeInContext(self.context, 64);
        let str_ty = LLVMStructTypeInContext(
            self.context,
            [LLVMPointerType(str_characters_ty, 0), len_ty].as_mut_ptr(),
            2,
            0,
        );

        let str_characters = LLVMAddGlobal(
            self.module,
            str_characters_ty,
            to_c_str(&format!("{}.characters", global_name)).as_ptr(),
        );
        LLVMSetGlobalConstant(str_characters, 1);
        LLVMSetInitializer(
            str_characters,
            LLVMConstStringInContext(self.context, to_c_str(&text).as_ptr(), text.len() as u32, 1),
        );

        let str = LLVMAddGlobal(self.module, str_ty, to_c_str(&global_name).as_ptr());
        LLVMSetGlobalConstant(str, 1);
        LLVMSetInitializer(
            str,
            LLVMConstNamedStruct(
                str_ty,
                [
                    LLVMConstBitCast(
                        str_characters,
                        LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
                    ),
                    LLVMConstInt(len_ty, text.len() as u64, 0),
                ]
                .as_mut_ptr(),
                2,
            ),
        );

        // Constants should be private
        LLVMSetLinkage(str_characters, LLVMLinkage::LLVMPrivateLinkage);
        LLVMSetLinkage(str, LLVMLinkage::LLVMPrivateLinkage);

        str
    }

    unsafe fn build_get_user_field(&self, field: Field) -> LLVMValueRef {
        match field {
            Field::Email => self.make_call("user_get_field_email", "email", &mut [self.user_arg]),
            Field::Gender => {
                self.make_call("user_get_field_gender", "gender", &mut [self.user_arg])
            }
            Field::PhoneNumber => self.make_call(
                "user_get_field_phone_number",
                "phone_number",
                &mut [self.user_arg],
            ),
            Field::LocationStreet => self.make_call(
                "user_get_field_location_street",
                "location_street",
                &mut [self.user_arg],
            ),
            Field::LocationCity => self.make_call(
                "user_get_field_location_city",
                "location_city",
                &mut [self.user_arg],
            ),
            Field::LocationState => self.make_call(
                "user_get_field_location_state",
                "location_state",
                &mut [self.user_arg],
            ),
            Field::Username => {
                self.make_call("user_get_field_username", "username", &mut [self.user_arg])
            }
            Field::Password => {
                self.make_call("user_get_field_password", "password", &mut [self.user_arg])
            }
            Field::FirstName => self.make_call(
                "user_get_field_first_name",
                "first_name",
                &mut [self.user_arg],
            ),
            Field::LastName => self.make_call(
                "user_get_field_last_name",
                "last_name",
                &mut [self.user_arg],
            ),
            Field::Title => self.make_call("user_get_field_title", "title", &mut [self.user_arg]),
            Field::Picture => {
                self.make_call("user_get_field_picture", "picture", &mut [self.user_arg])
            }
        }
    }

    unsafe fn build_filter(&mut self, filter: &Filter) -> LLVMValueRef {
        let text = self.build_global_str(&filter.value);
        let str = self.make_call("separated_str_as_str", "str", &mut [text]);

        let field = self.build_get_user_field(filter.field);

        match filter.kind {
            crate::FilterKind::StrContains => {
                self.make_call("filter_str_contains", "contains", &mut [field, str])
            }
            crate::FilterKind::StrEquals => {
                self.make_call("filter_str_equals", "equals", &mut [field, str])
            }
            crate::FilterKind::StrStartsWith => {
                self.make_call("filter_str_starts_with", "starts_with", &mut [field, str])
            }
            crate::FilterKind::StrEndsWith => {
                self.make_call("filter_str_ends_with", "ends_with", &mut [field, str])
            }
        }
    }

    unsafe fn build_join_filter(
        &mut self,
        filter: &JoinFilters,
        fail_block: LLVMBasicBlockRef,
        success_block: LLVMBasicBlockRef,
    ) {
        match filter {
            JoinFilters::Filter(f) => {
                let result = self.build_filter(f);
                // Build br
                LLVMBuildCondBr(self.builder, result, success_block, fail_block);
            }
            JoinFilters::And(left, right) => {
                let and_middle_block = LLVMAppendBasicBlockInContext(
                    self.context,
                    LLVMGetBasicBlockParent(LLVMGetInsertBlock(self.builder)),
                    to_c_str("and_middle").as_ptr(),
                );

                self.build_join_filter(left, fail_block, and_middle_block);
                LLVMPositionBuilderAtEnd(self.builder, and_middle_block);
                self.build_join_filter(right, fail_block, success_block);
            }
            JoinFilters::Or(left, right) => {
                let or_middle_block = LLVMAppendBasicBlockInContext(
                    self.context,
                    LLVMGetBasicBlockParent(LLVMGetInsertBlock(self.builder)),
                    to_c_str("or_middle").as_ptr(),
                );

                self.build_join_filter(left, or_middle_block, success_block);
                LLVMPositionBuilderAtEnd(self.builder, or_middle_block);
                self.build_join_filter(right, fail_block, success_block);
            }
        }
    }
}

pub unsafe fn build_fn(
    name: &str,
    module: LLVMModuleRef,
    context: LLVMContextRef,
    filters: &JoinFilters,
) {
    build_filter_fn("filter", module, context, filters);

    // Grab the function signature we want to copy and add it to the module
    let fn_val = LLVMGetNamedFunction(module, to_c_str("fn_sig").as_ptr());
    let fn_type = LLVMGlobalGetValueType(fn_val);
    let fn_value = LLVMAddFunction(module, to_c_str(name).as_ptr(), fn_type);

    let entry_block = LLVMAppendBasicBlockInContext(context, fn_value, to_c_str("entry").as_ptr());
    let users_arr_arg = LLVMGetParam(fn_value, 0);
    let result_vec_arg = LLVMGetParam(fn_value, 1);

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, entry_block);

    let make_call = |fn_name: &str, result_name: &str, args: &mut [*mut LLVMValue]| {
        let f = LLVMGetNamedFunction(module, to_c_str(fn_name).as_ptr());

        if f.is_null() {
            panic!("Function {} not found", fn_name);
        }

        let ty = LLVMGlobalGetValueType(f);

        LLVMBuildCall2(
            builder,
            ty,
            f,
            args.as_mut_ptr(),
            args.len() as u32,
            to_c_str(result_name).as_ptr(),
        )
    };

    let filter_fn_ptr = LLVMGetNamedFunction(module, to_c_str("filter").as_ptr());

    make_call(
        "run_filter",
        "result",
        &mut [users_arr_arg, result_vec_arg, filter_fn_ptr],
    );

    // Return
    LLVMBuildRetVoid(builder);

    LLVMDisposeBuilder(builder);
}
