use pest::{iterators::Pair, Parser};
use ppl_ast::{
    declarations::{Function, FunctionId, Identifier, Parameter, Text, Typename},
    module::Module,
};

use crate::{
    diagnostic::{Diagnostic, Diagnostics},
    source::SourceProgram,
    Db,
};

#[derive(pest_derive::Parser)]
#[grammar = "ppl.pest"]
struct PPLParser;

#[salsa::tracked]
pub fn parse_module(db: &dyn Db, source: SourceProgram) -> Module {
    let source_text = source.text(db);

    let module = PPLParser::parse(Rule::module, &source_text);
    if let Err(err) = module {
        let err = if let Some(path) = source.path(db) {
            err.with_path(path.to_str().unwrap())
        } else {
            err
        };
        Diagnostics::push(
            db,
            Diagnostic {
                message: err.to_string(),
            },
        );
        return Module::new(db, vec![]);
    }
    let module = module.unwrap().next().unwrap();

    let mut functions: Vec<Function> = Vec::new();
    for f in module.into_inner() {
        match f.as_rule() {
            Rule::function => {
                functions.push(function(db, f));
            }
            _ => {}
        }
    }

    Module::new(db, functions)
}

fn function(db: &dyn Db, tree: Pair<'_, Rule>) -> Function {
    let name_parts: Vec<_> = tree
        .into_inner()
        .filter_map(|part| {
            Some(match part.as_rule() {
                Rule::text => Text::new(db, part.as_str().to_string()).into(),
                Rule::parameter => parameter(db, part).into(),
                _ => return None,
            })
        })
        .collect();
    Function::new(db, FunctionId::from_parts(db, &name_parts), name_parts)
}

fn parameter(db: &dyn Db, tree: Pair<'_, Rule>) -> Parameter {
    let mut parts = tree.into_inner();
    Parameter {
        name: identifier(db, parts.next().unwrap()),
        ty: typename(db, parts.next().unwrap()),
    }
}

fn identifier(db: &dyn Db, tree: Pair<'_, Rule>) -> Identifier {
    Identifier::new(db, tree.as_str().to_string())
}

fn typename(db: &dyn Db, tree: Pair<'_, Rule>) -> Typename {
    Typename::new(db, tree.as_str().to_string())
}
