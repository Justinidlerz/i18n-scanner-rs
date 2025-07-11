use crate::node::i18n_types::I18nMember;
use crate::node::node::Node;
use crate::node::node_store::NodeStore;
use crate::walk_utils::WalkerUtils;
use oxc_ast::ast::{CallExpression, Expression, Statement, StringLiteral, VariableDeclarator};
use oxc_ast::AstKind;
use oxc_resolver::Resolver;
use oxc_semantic::Semantic;
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct Walker<'a> {
  resolver: Rc<Resolver>,
  node: Rc<Node>,
  externals: Rc<Vec<Regex>>,
  // reexport all members
  reexport_all_importing: Vec<String>,
  // { exports: [], file_paths: '' }[]
  i18n_methods: NodeStore,
  // To collect members and replace the file_path as pointer
  importing_collection: HashMap<String, String>,
  semantic: &'a Semantic<'a>,
  pub walk_utils: WalkerUtils<'a>,
}

impl<'a> Walker<'a> {
  pub fn new(
    resolver: Rc<Resolver>,
    node: Rc<Node>,
    i18n_methods: NodeStore,
    semantic: &'a Semantic<'a>,
    externals: Rc<Vec<Regex>>,
  ) -> Self {
    Self {
      externals,
      resolver,
      node: node.clone(),
      semantic,
      reexport_all_importing: Vec::new(),
      i18n_methods,
      importing_collection: HashMap::new(),
      walk_utils: WalkerUtils::new(semantic, node),
    }
  }

  pub fn append_reexport(&mut self, source: &StringLiteral) {
    self.reexport_all_importing.push(source.value.to_string());
  }

  pub fn append_exports(&mut self, members: Vec<(String, Option<I18nMember>)>) {
    for (name, i18n_member) in members {
      self.node.insert_exporting(name, i18n_member);
    }
  }

  pub fn resolve_import(&mut self, source: &StringLiteral, specifiers: Vec<String>) {
    // external packages no need to be collected
    if self
      .externals
      .iter()
      .any(|reg| reg.is_match(source.value.as_str()))
    {
      return;
    }

    let basename = Path::new(self.node.file_path.as_str())
      .parent()
      .unwrap_or_else(|| Path::new("."));

    if let Ok(res) = self
      .resolver
      .resolve(basename.to_str().unwrap(), source.value.as_str())
    {
      if let Some(path_str) = res.path().to_str() {
        if let Some(node) = self.i18n_methods.get_node(path_str) {
          let importing_node_members = node.get_exporting_i18n_members();

          if specifiers.iter().any(|specifier| {
            // is matched i18n methods or
            // namespace import, no matter is reexport all or not
            importing_node_members.contains_key(specifier) || specifier == "*"
          }) {
            self.node.mark_has_i18n_source_imported();
          }
        }

        self
          .node
          .try_insert_importing(source.value.to_string(), path_str.to_string())
          .unwrap_or_else(|_| {
            self
              .importing_collection
              .entry(source.value.to_string())
              .or_insert_with(|| path_str.to_string());
          })
      } else {
        println!("[i18n-scanner-rs] failed to format path: {}", source.value)
      }
    } else {
      println!(
        "[i18n-scanner-rs] failed to resolve: {} in {}",
        source.value, self.node.file_path
      );
    }
  }

  pub fn get_importing_collection(&self) -> HashMap<String, String> {
    self.importing_collection.clone()
  }

  pub fn resolve_i18n_export(&self, de: &VariableDeclarator) -> Option<I18nMember> {
    // import should before export, and it should import i18n source
    if !self.node.has_i18n_source_imported() {
      return None;
    }
    let Some(init) = &de.init else {
      return None;
    };

    let resolve_expression_fn = |state: &Statement| -> Option<I18nMember> {
      let Statement::ExpressionStatement(exp) = state else {
        return None;
      };
      let Expression::CallExpression(call) = &exp.expression else {
        return None;
      };

      self.resolve_call_i18n_method(call)
    };

    let resolve_return_fn = |state: &Statement| -> Option<I18nMember> {
      let Statement::ReturnStatement(ret) = state else {
        return None;
      };
      let Some(argument) = &ret.argument else {
        return None;
      };
      let Expression::CallExpression(call) = argument else {
        return None;
      };
      self.resolve_call_i18n_method(call)
    };

    match &init {
      Expression::ArrowFunctionExpression(func) => {
        // Nothing implemented
        // TODO: handle other cases
        if func.body.statements.len() != 1 {
          return None;
        };

        // const fn = () => useTranslation("abc");
        if func.expression {
          return resolve_expression_fn(&func.body.statements[0]);
        }

        resolve_return_fn(&func.body.statements[0])
      }
      // const a = function () {}
      Expression::FunctionExpression(func) => {
        // Nothing implemented
        let Some(body) = &func.body else {
          return None;
        };
        // TODO: handle other cases
        if body.statements.len() != 1 {
          return None;
        };

        if func.is_expression() {
          return self.resolve_expression_fn(&body.statements[0]);
        }
        resolve_return_fn(&body.statements[0])
      }
      _ => None,
    }
  }

  pub fn resolve_expression_fn(&self, state: &Statement) -> Option<I18nMember> {
    let Statement::ExpressionStatement(exp) = state else {
      return None;
    };
    let Expression::CallExpression(call) = &exp.expression else {
      return None;
    };
    self.resolve_call_i18n_method(call)
  }
  
  pub fn resolve_call_i18n_method(&self, call_exp: &CallExpression) -> Option<I18nMember> {
    let Expression::Identifier(id) = &call_exp.callee else {
      return None;
    };

    let r#ref = self
      .semantic
      .scoping()
      .get_reference(id.reference_id.get().unwrap());
    let node = self.semantic.symbol_declaration(r#ref.symbol_id().unwrap());

    let spec = match node.kind() {
      AstKind::ImportSpecifier(spec) => self
        .semantic
        .nodes()
        .parent_node(node.id())
        .and_then(|node| Some((spec.imported.name().to_string(), node))),
      _ => None,
    };

    let member = spec
      .and_then(|(spec, ast_node)| match ast_node.kind() {
        AstKind::ImportDeclaration(decl) => self
          .node
          .get_importing_node(&decl.source.value.to_string())
          .and_then(|node| Some((spec, node))),
        _ => None,
      })
      .and_then(|(spec, node)| {
        let members = node.get_exporting_i18n_members();
        if let Some(member) = members.get(&spec) {
          let ns = self.walk_utils.read_hook_namespace_argument(&call_exp);
          return Some(I18nMember {
            r#type: member.r#type.clone(),
            ns,
          });
        }
        None
      });

    member
  }
}
