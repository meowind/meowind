use std::collections::HashMap;

use cranelift::prelude::{EntityRef, FunctionBuilder, InstBuilder, Type, Variable};
use cranelift_module::Module;

use crate::{errors::compiler::CompilerError, frontend::parsing::ast::functions::FunctionNode};

pub struct FunctionTranslator<'a, T: Module> {
    pub errors: Vec<CompilerError<'a>>,

    isize: Type,
    module: &'a mut T,
    pub(super) builder: FunctionBuilder<'a>,

    variables: HashMap<String, Variable>,
    // TODO: pass parent variables to arguments
    ast: &'a FunctionNode,
}

impl<'a, T: Module> FunctionTranslator<'a, T> {
    pub fn new(module: &'a mut T, builder: FunctionBuilder<'a>, ast: &'a FunctionNode) -> Self {
        let isize = module.target_config().pointer_type();

        Self {
            errors: Vec::new(),

            isize,
            module,
            builder,

            variables: HashMap::new(),
            ast,
        }
    }

    pub fn translate(&mut self) {
        let entry_block = self.builder.create_block();
        self.builder
            .append_block_params_for_function_params(entry_block);
        self.builder.switch_to_block(entry_block);
        self.builder.seal_block(entry_block);

        for arg in &self.ast.args {
            self.declare_variable(&arg.name, self.isize);
        }

        let val = self.builder.ins().iconst(self.isize, 50);
        self.builder.ins().return_(&[val]);
    }

    fn declare_variable(&mut self, name: &str, r#type: Type) -> Variable {
        let var = Variable::new(self.variables.len());

        if !self.variables.contains_key(name) {
            self.variables.insert(name.into(), var);
            self.builder.declare_var(var, r#type);
        }
        return var;
    }
}
