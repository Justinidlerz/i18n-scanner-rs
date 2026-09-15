use crate::analyzer::i18n_packages::{is_preset_member_name, preset_member_type};
use crate::node::i18n_types::{I18nMember, I18nType};
use crate::node::node::{ExportBinding, Node};
use crate::node::node_store::NodeStore;
use crate::walk_utils::WalkerUtils;
use log::debug;
use oxc_ast::ast::{
  CallExpression, Expression, FunctionBody, IdentifierReference, Statement, StringLiteral,
  VariableDeclarator,
};
use oxc_ast::AstKind;
use oxc_resolver::Resolver;
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct Walker<'a> {
  resolver: Rc<Resolver>,
  pub collect_exports: bool,
  pub has_exports: bool,
  node: Rc<Node>,
  externals: Rc<Vec<Regex>>,
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
      collect_exports: false,
      has_exports: false,
      externals,
      resolver,
      node: node.clone(),
      semantic,
      i18n_methods,
      importing_collection: HashMap::new(),
      walk_utils: WalkerUtils::new(semantic, node),
    }
  }

  fn apply_resolved_import(
    &mut self,
    source: &StringLiteral,
    specifiers: &[String],
    path_str: String,
    is_external: bool,
  ) {
    if is_external && self.i18n_methods.get_node(&path_str).is_none() {
      return;
    }

    self
      .node
      .insert_importing_specifiers(source.value.to_string(), specifiers);

    self
      .node
      .try_insert_importing(source.value.to_string(), path_str.clone())
      .unwrap_or_else(|_| {
        self
          .importing_collection
          .entry(source.value.to_string())
          .or_insert(path_str);
      });
  }

  pub fn append_reexport(&mut self, source: &StringLiteral) {
    self.node.insert_reexport_all(source.value.to_string());
  }

  pub fn append_exports(&mut self, members: Vec<(String, ExportBinding)>) {
    for (name, i18n_member) in members {
      self.node.insert_export_binding(name, i18n_member);
    }
  }

  pub fn resolve_import(&mut self, source: &StringLiteral, specifiers: Vec<String>) {
    // Export resolution reads the completed graph; it must not rebuild its edges.
    if self.collect_exports {
      return;
    }
    let is_external = self
      .externals
      .iter()
      .any(|reg| reg.is_match(source.value.as_str()));

    match self
      .resolver
      .resolve_file(self.node.file_path.as_str(), source.value.as_str())
    {
      Ok(res) => {
        if let Some(path_str) = res.path().to_str() {
          self.apply_resolved_import(source, &specifiers, path_str.to_string(), is_external);
        } else {
          debug!("[i18n-scanner-rs] failed to format path: {}", source.value)
        }
      }
      Err(err) => {
        debug!(
          "[i18n-scanner-rs] failed to resolve: {} in {} ({})",
          source.value, self.node.file_path, err
        );
      }
    }
  }

  pub fn get_importing_collection(&self) -> HashMap<String, String> {
    self.importing_collection.clone()
  }

  pub fn is_custom_i18n_hook_function(&self, body: &FunctionBody) -> bool {
    // Check if this function uses useTranslation and returns a t() call
    // This would make it a custom i18n hook

    let mut uses_use_translation = false;
    let mut returns_t_call = false;

    for stmt in &body.statements {
      // Check for useTranslation call
      if self.statement_contains_use_translation(stmt) {
        uses_use_translation = true;
      }

      // Check for return statement with t() call
      if let Statement::ReturnStatement(ret_stmt) = stmt {
        if let Some(arg) = &ret_stmt.argument {
          if self.expression_contains_t_call(arg) {
            returns_t_call = true;
          }
        }
      }
    }

    uses_use_translation && returns_t_call
  }

  fn statement_contains_use_translation(&self, stmt: &Statement) -> bool {
    match stmt {
      Statement::VariableDeclaration(var_decl) => {
        for declarator in &var_decl.declarations {
          if let Some(init) = &declarator.init {
            if self.expression_contains_use_translation(init) {
              return true;
            }
          }
        }
      }
      _ => {}
    }
    false
  }

  fn expression_contains_use_translation(&self, expr: &Expression) -> bool {
    match expr {
      Expression::CallExpression(call) => {
        if let Expression::Identifier(ident) = &call.callee {
          let hook_type = I18nType::Hook;
          return is_preset_member_name(ident.name.as_str(), &hook_type);
        }
      }
      _ => {}
    }
    false
  }

  fn expression_contains_t_call(&self, expr: &Expression) -> bool {
    match expr {
      Expression::CallExpression(call) => {
        if let Expression::Identifier(ident) = &call.callee {
          let t_method_type = I18nType::TMethod;
          return is_preset_member_name(ident.name.as_str(), &t_method_type);
        }
      }
      _ => {}
    }
    false
  }

  pub fn resolve_i18n_export(&self, de: &VariableDeclarator) -> ExportBinding {
    self.resolve_variable_export(de, &mut HashSet::new())
  }

  fn resolve_variable_export(
    &self,
    de: &VariableDeclarator,
    visiting: &mut HashSet<SymbolId>,
  ) -> ExportBinding {
    let fallback = de.id.get_binding_identifier().and_then(|ident| {
      preset_member_type(ident.name.as_str()).map(|r#type| I18nMember {
        r#type,
        ns: None,
        key_prop: None,
      })
    });
    let Some(init) = &de.init else {
      return ExportBinding::Local(fallback);
    };
    let resolved = match init {
      Expression::ArrowFunctionExpression(func) => {
        if func.body.statements.len() == 1 {
          self.resolve_wrapper_statement(&func.body.statements[0], visiting)
        } else if self.is_custom_i18n_hook_function(&func.body) {
          Some(ExportBinding::Local(Some(I18nMember {
            r#type: I18nType::Hook,
            ns: None,
            key_prop: None,
          })))
        } else {
          None
        }
      }
      Expression::FunctionExpression(func) => func.body.as_ref().and_then(|body| {
        if body.statements.len() == 1 {
          self.resolve_wrapper_statement(&body.statements[0], visiting)
        } else {
          None
        }
      }),
      _ => self.resolve_export_expression(init, visiting),
    };
    resolved.unwrap_or(ExportBinding::Local(fallback))
  }

  fn resolve_wrapper_statement(
    &self,
    statement: &Statement,
    visiting: &mut HashSet<SymbolId>,
  ) -> Option<ExportBinding> {
    let expression = match statement {
      Statement::ExpressionStatement(statement) => &statement.expression,
      Statement::ReturnStatement(statement) => statement.argument.as_ref()?,
      _ => return None,
    };
    let Expression::CallExpression(call) = expression else {
      return None;
    };
    self.resolve_call_i18n_method(call, visiting)
  }

  pub fn resolve_export_identifier(&self, ident: &IdentifierReference) -> ExportBinding {
    self
      .resolve_identifier(ident, &mut HashSet::new())
      .unwrap_or(ExportBinding::Local(None))
  }

  fn resolve_identifier(
    &self,
    ident: &IdentifierReference,
    visiting: &mut HashSet<SymbolId>,
  ) -> Option<ExportBinding> {
    let reference_id = ident.reference_id.get()?;
    let symbol_id = self
      .semantic
      .scoping()
      .get_reference(reference_id)
      .symbol_id()?;
    if !visiting.insert(symbol_id) {
      return None;
    }
    let node = self.semantic.symbol_declaration(symbol_id);
    let imported = match node.kind() {
      AstKind::ImportSpecifier(spec) => Some(spec.imported.name().to_string()),
      AstKind::ImportDefaultSpecifier(_) => Some("default".into()),
      _ => None,
    };
    let result = if let Some(name) = imported {
      match self.semantic.nodes().parent_node(node.id()).kind() {
        AstKind::ImportDeclaration(decl) => Some(ExportBinding::Imported {
          source: decl.source.value.to_string(),
          name,
        }),
        _ => None,
      }
    } else {
      match node.kind() {
        AstKind::VariableDeclarator(decl) => Some(ExportBinding::LocalAlias {
          name: decl.id.get_binding_identifier()?.name.to_string(),
          binding: Box::new(self.resolve_variable_export(decl, visiting)),
        }),
        AstKind::Function(func) => func.id.as_ref().and_then(|id| {
          preset_member_type(id.name.as_str()).map(|r#type| ExportBinding::LocalAlias {
            name: id.name.to_string(),
            binding: Box::new(ExportBinding::Local(Some(I18nMember {
              r#type,
              ns: None,
              key_prop: None,
            }))),
          })
        }),
        _ => None,
      }
    };
    visiting.remove(&symbol_id);
    result
  }

  pub fn resolve_export_expression(
    &self,
    expression: &Expression,
    visiting: &mut HashSet<SymbolId>,
  ) -> Option<ExportBinding> {
    match expression {
      Expression::Identifier(ident) => self.resolve_identifier(ident, visiting),
      Expression::StaticMemberExpression(member) => {
        let Expression::Identifier(ident) = &member.object else {
          return None;
        };
        let node = self
          .walk_utils
          .get_var_defined_node(ident.reference_id.get()?)?;
        if !matches!(node.kind(), AstKind::ImportNamespaceSpecifier(_)) {
          return None;
        }
        let AstKind::ImportDeclaration(decl) = self.semantic.nodes().parent_node(node.id()).kind()
        else {
          return None;
        };
        Some(ExportBinding::Imported {
          source: decl.source.value.to_string(),
          name: member.property.name.to_string(),
        })
      }
      _ => None,
    }
  }

  fn resolve_call_i18n_method(
    &self,
    call: &CallExpression,
    visiting: &mut HashSet<SymbolId>,
  ) -> Option<ExportBinding> {
    let callee = self.resolve_export_expression(&call.callee, visiting)?;
    Some(ExportBinding::Wrapper {
      callee: Box::new(callee),
      ns: self.walk_utils.read_hook_namespace_argument(call),
    })
  }
}
