use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::types::{PointerType, VoidType};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use crate::compiler::parser::CodeElement;

const ARRAY_SIZE: i32 = 1<<10;
pub struct Codegen<'ctx>
{
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    array: PointerValue<'ctx>,
    pointer: PointerValue<'ctx>,

    print: FunctionValue<'ctx>,
    exit: FunctionValue<'ctx>,

    main_fn: FunctionValue<'ctx>
}

impl<'ctx> Codegen<'ctx>
{
    pub fn codegen(&self, to_gen: impl Iterator<Item=CodeElement>) -> Vec<u8>
    {
        self.codegen_internal(to_gen);
        self.output_as_object()
    }

    fn output_as_object(&self) -> Vec<u8>
    {
        Target::initialize_all(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .expect("Unable to find LLVM target for the given triple");
        let cpu = "generic";
        let features = "";
        let opt_level = OptimizationLevel::Default;
        let reloc_mode = RelocMode::Default;
        let code_model = CodeModel::Default;

        let target_machine = target
            .create_target_machine(
                &triple,
                cpu,
                features,
                opt_level,
                reloc_mode,
                code_model,
            )
            .expect("Unable to create target machine");

        let buffer = target_machine
            .write_to_memory_buffer(&self.module, FileType::Object)
            .expect("Failed to emit object to memory");
        buffer.as_slice().to_vec()
    }

    fn codegen_internal(&self, to_gen: impl Iterator<Item=CodeElement> + Sized) {
        for element in to_gen
        {
            self.codegen_single(element)
        }
        self.builder.build_call(self.exit, &[], "exit").unwrap();
        self.builder.build_return(None).unwrap();
    }

    fn codegen_single(&self, to_gen: CodeElement)
    {
        match to_gen
        {
            CodeElement::IncPtr => self.codegen_ptr_inc(),
            CodeElement::DecPtr => self.codegen_ptr_dec(),
            CodeElement::Inc => self.codegen_inc(),
            CodeElement::Dec => self.codegen_dec(),
            CodeElement::Print => self.codegen_print(),
            CodeElement::Read => todo!(),
            CodeElement::Loop(elements) => self.codegen_loop(elements)
        }
    }

    fn codegen_print(&self)
    {
        let to_print = self.codegen_get_pointer_to_selected_value();
        self.builder.build_call(self.print, &[to_print.into()], "print").unwrap();
    }

    fn codegen_loop(& self, inner: Vec<CodeElement>)
    {
        let parent = self.main_fn;
        let loop_cond_bb = self.context.append_basic_block(parent, "loop_cond");

        self.builder.build_unconditional_branch(loop_cond_bb).unwrap();
        let loop_bb = self.context.append_basic_block(parent, "loop");
        self.builder.position_at_end(loop_bb);
        for inner_element in inner
        {
            self.codegen_single(inner_element);
        }
        self.builder.build_unconditional_branch(loop_cond_bb).unwrap();
        let after_loop_bb = self.context.append_basic_block(parent, "after_loop");
        self.builder.position_at_end(loop_cond_bb);

        let address = self.codegen_get_pointer_to_selected_value();
        self.builder.build_conditional_branch(self.builder.build_int_compare(
            IntPredicate::EQ,
            self.codegen_load_specified_ptr(address),
            self.context.i32_type().const_zero(), "cond_load").unwrap(), after_loop_bb, loop_bb).unwrap();
        self.builder.position_at_end(after_loop_bb);
    }

    fn codegen_ptr_inc(&self)
    {
        self.codegen_change_ptr(1);
    }

    fn codegen_ptr_dec(&self)
    {
        self.codegen_change_ptr(-1);
    }

    fn codegen_inc(&self)
    {
        self.codegen_change_value(1);
    }

    fn codegen_dec(&self)
    {
        self.codegen_change_value(-1);
    }

    fn codegen_change_ptr(&self, amount: i32)
    {
        self.add_to_memory(amount, self.pointer);
    }

    fn codegen_change_value(&self, amount: i32)
    {
        let address = self.codegen_get_pointer_to_selected_value();
        self.add_to_memory(amount, address);
    }

    fn add_to_memory(&self, amount: i32, address: PointerValue) {
        self.builder.build_store(
            address,
            self.builder.build_int_add(
                self.codegen_load_specified_ptr(address),
                self.context.i32_type().const_int(amount as u64, true), "add").unwrap()).unwrap();
    }

    fn codegen_get_pointer_to_selected_value(&self) -> PointerValue
    {
        let location = self.codegen_load_specified_ptr(self.pointer);
        unsafe
            {
                self.builder.build_gep(
                    self.context.i32_type(),
                    self.array,
                    &[
                        location
                    ],
                    "value_pointer_create"
                ).unwrap()
            }
    }

    fn codegen_load_specified_ptr<'b>(&self, to_load: PointerValue<'b>) -> IntValue<'b>
    where 'ctx: 'b
    {
        self.builder.build_load(
            self.context.i32_type(),
            to_load, "load")
            .unwrap().into_int_value()
    }

    pub fn new(context: &'ctx Context, builder: Builder<'ctx>, module: Module<'ctx>) -> Self
    {
        let main_fn = module.add_function("main", context.void_type().fn_type(&[], false), None);
        let main_bb = context.append_basic_block(main_fn, "main");

        let void_type = context.void_type();
        let ptr_type = context.ptr_type(AddressSpace::from(0));

        let print_fn = Self::load_print(context, &module, void_type, ptr_type);
        let zero_mem_fn = Self::load_zero_mem(context, &module, void_type, ptr_type);
        let exit_fn = Self::load_exit(&module, void_type);

        builder.position_at_end(main_bb);
        let array_size = context.i32_type().const_int(ARRAY_SIZE as u64, true);
        let array = builder
            .build_array_alloca(
                context.i32_type(),
                array_size,
                "array")
            .unwrap();
        let pointer = builder
            .build_alloca(
                context.i32_type(),
                "pointer")
            .unwrap();
        builder.build_store(
            pointer,
            context.i32_type().const_int(0, false))
            .unwrap();


        let temp_self = Self { context, builder, module, array, pointer, main_fn, print: print_fn, exit: exit_fn};

        let array_size_bytes = context.i32_type().const_int(ARRAY_SIZE as u64*4, true);
        temp_self.builder.build_call(zero_mem_fn, &[pointer.into(), array_size_bytes.into()], "zero_array").unwrap();
        temp_self
    }

    fn load_exit<'b>(module: &Module<'b>, void_type: VoidType<'b>) -> FunctionValue<'b> {
        let exit_type = void_type.fn_type(&[], false);
        let exit_fn: FunctionValue = module.add_function("exit", exit_type, None);
        exit_fn.set_linkage(inkwell::module::Linkage::External);
        exit_fn
    }

    fn load_zero_mem<'b>(context: &'b Context, module: &Module<'b>, void_type: VoidType<'b>, ptr_type: PointerType<'b>) -> FunctionValue<'b> {
        let i64_type = context.i64_type();
        let zero_mem_type = void_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
        let zero_mem_fn: FunctionValue = module.add_function("zero_mem", zero_mem_type, None);
        zero_mem_fn.set_linkage(inkwell::module::Linkage::External);
        zero_mem_fn
    }

    fn load_print<'b>(context: &'b Context, module: &Module<'b>, void_type: VoidType<'b>, ptr_type: PointerType<'b>) -> FunctionValue<'b> {

        let print_type = void_type.fn_type(&[ptr_type.into()], false);
        let print_fn: FunctionValue = module.add_function("print", print_type, None);
        print_fn
    }
}

pub fn codegen(to_gen: impl Iterator<Item=CodeElement>) -> Vec<u8>
{
    let context = Context::create();
    let builder = context.create_builder();
    let module = context.create_module("main");

    let codegen = Codegen::new(&context, builder, module);
    codegen.codegen(to_gen)
}