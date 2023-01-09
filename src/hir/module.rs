use std::collections::{HashMap, HashSet};

use crate::ast;
use crate::hir::{FunctionDeclaration, Statement, TypeDeclaration, VariableDeclaration};
use crate::named::{HashByName, Named};
use crate::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use std::sync::{Arc, LazyLock};

/// Module with PPL code
#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    /// Name of the module
    pub name: String,

    /// Variables, declared in this module
    pub variables: HashSet<HashByName<Arc<VariableDeclaration>>>,

    /// Types, declared in this module
    pub types: HashSet<HashByName<Arc<TypeDeclaration>>>,

    /// Functions, declared in this module
    pub functions: HashMap<
        String, // Name format
        HashSet<HashByName<Arc<FunctionDeclaration>>>,
    >,

    /// Statements in this module
    pub statements: Vec<Statement>,
}

static BUILTIN: LazyLock<Arc<Module>> =
	LazyLock::new(|| Arc::new(Module::create_builtin()));

impl Module {
    /// Create an empty module
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            variables: HashSet::new(),
            types: HashSet::new(),
            functions: HashMap::new(),
            statements: vec![],
        }
    }

    /// Create builtin module
    fn create_builtin_from_str(content: &str) -> miette::Result<Self> {
        let ast = content.parse::<ast::Module>()?;

        let mut context = ASTLoweringContext {
            module: Module::new("ppl"),
            builtin: None,
			functions_stack: vec![],
        };

        for stmt in ast.statements {
            stmt.lower_to_hir_within_context(&mut context)?;
        }

        Ok(context.module)
    }

    /// Create builtin module
    pub(crate) fn create_builtin() -> Self {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/ppl.ppl");

        let content =
            std::fs::read_to_string(path).expect(format!("Failed to read {}", path).as_str());

        let module = Self::create_builtin_from_str(&content);
        if let Err(err) = module {
            panic!(
                "Error in builtin module: {:?}",
                err.with_source_code(miette::NamedSource::new("ppl.ppl", content))
            );
        }

        module.unwrap()
    }

    /// Get builtin module
    ///
    /// # Example
    /// ```
    /// use ppl::hir::Module;
    ///
    /// let module = Module::builtin();
    /// ```
    pub fn builtin() -> Arc<Self> {
        BUILTIN.clone()
    }

	/// Insert function to module
	pub fn insert_function(&mut self, function: Arc<FunctionDeclaration>) {
		let set =
			self.functions
				.entry(function.name_format().to_string())
				.or_insert_with(HashSet::new);
		set.insert(function.into());
	}
}

impl Named for Module {
    fn name(&self) -> &str {
        &self.name
    }
}
