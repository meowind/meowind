use cranelift::{
    codegen::{
        ir::{Function, UserFuncName},
        Context,
    },
    prelude::{isa::CallConv, AbiParam, FunctionBuilder, FunctionBuilderContext, Signature, Type},
};
use cranelift_module::{Linkage, Module, ModuleError};
use functions::FunctionTranslator;

use crate::{
    errors::compiler::CompilerError,
    frontend::parsing::ast::{
        functions::FunctionNode,
        items::{ItemKind, ItemNode},
        namespaces::NamespaceNode,
        projects::ProjectNode,
    },
};

pub mod functions;

pub struct Translator<'a, T: Module> {
    pub module: T,
    pub errors: Vec<CompilerError<'a>>,
    pub ctx: Context,

    ast: &'a ProjectNode,
    pub(self) isize: Type,

    pub(self) func_builder_ctx: FunctionBuilderContext,
}

impl<'a, T: Module> Translator<'a, T> {
    pub fn new(module: T, ast: &'a ProjectNode) -> Self {
        let isize = module.target_config().pointer_type();
        let ctx = module.make_context();

        Self {
            errors: Vec::new(),
            ctx,

            module,
            ast,
            isize,

            func_builder_ctx: FunctionBuilderContext::new(),
        }
    }

    pub fn translate(module: T, ast: &'a ProjectNode) -> Self {
        let mut trans = Self::new(module, ast);
        trans.translate_project();

        return trans;
    }

    fn translate_project(&mut self) {
        self.translate_namespace(&self.ast.root);
    }

    fn translate_namespace(&mut self, namespace: &NamespaceNode) {
        for item in &namespace.items {
            self.translate_item(item);
        }
    }

    fn translate_item(&mut self, item: &ItemNode) {
        match &item.kind {
            ItemKind::Function(node) => {
                self.translate_function(node);
            }
            _ => todo!(),
        }
    }

    fn translate_function(&mut self, node: &FunctionNode) {
        // TODO: more types
        let mut sig = Signature::new(CallConv::SystemV);

        for _arg in &node.args {
            sig.params.push(AbiParam::new(self.isize));
        }

        sig.returns.push(AbiParam::new(self.isize));
        let mut func = Function::with_name_signature(UserFuncName::user(0, 0), sig);

        let builder = FunctionBuilder::new(&mut func, &mut self.func_builder_ctx);
        let mut trans = FunctionTranslator::new(&mut self.module, builder, node);
        trans.translate();

        trans.builder.finalize();
        let func_id = self
            .module
            .declare_function(&node.name, Linkage::Local, &func.signature)
            .unwrap();

        self.module
            .define_function(func_id, &mut Context::for_function(func))
            .map_err(|err| {});
    }
}
