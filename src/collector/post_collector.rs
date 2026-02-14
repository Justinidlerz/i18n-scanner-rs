use crate::node::node_store::NodeStore;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
  BindingPatternKind, Declaration, ExportDefaultDeclarationKind, Expression,
  ImportDeclarationSpecifier, SourceType, Statement,
};
use oxc_parser::Parser;
use oxc_resolver::Resolver;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct PendingIdentifierKey {
  pub node_path: String,
  pub namespace: String,
  pub identifier: String,
}

pub struct PostCollector {
  pending_identifier_keys: Vec<PendingIdentifierKey>,
}

impl PostCollector {
  pub fn new() -> Self {
    Self {
      pending_identifier_keys: vec![],
    }
  }

  pub fn add_pending_identifier_key(
    &mut self,
    node_path: String,
    namespace: String,
    identifier: String,
  ) {
    self.pending_identifier_keys.push(PendingIdentifierKey {
      node_path,
      namespace,
      identifier,
    });
  }

  pub fn resolve_pending_keys(&self, node_store: &NodeStore) -> HashMap<String, Vec<String>> {
    let mut resolved = HashMap::<String, Vec<String>>::new();

    for pending in &self.pending_identifier_keys {
      let mut visited = HashSet::new();
      if let Some(value) = Self::resolve_identifier_in_file(
        node_store,
        &pending.node_path,
        &pending.identifier,
        &mut visited,
      ) {
        resolved
          .entry(pending.namespace.clone())
          .or_default()
          .push(value);
      }
    }

    resolved
  }

  fn resolve_identifier_in_file(
    node_store: &NodeStore,
    file_path: &str,
    identifier: &str,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    let visit_key = format!("{}::{}", file_path, identifier);
    if !visited.insert(visit_key) {
      // Break cycles like `const a = b; const b = a` across files.
      return None;
    }

    let source_text = std::fs::read_to_string(file_path).ok()?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).ok()?;
    let parser = Parser::new(&allocator, &source_text, source_type);
    let program = parser.parse().program;

    // Resolve local/imported bindings first because `t(KEY)` references local symbols.
    for stmt in &program.body {
      match stmt {
        Statement::ImportDeclaration(import_decl) => {
          if let Some(specifiers) = &import_decl.specifiers {
            for specifier in specifiers {
              if let ImportDeclarationSpecifier::ImportSpecifier(import_spec) = specifier {
                if import_spec.local.name == identifier {
                  let source = import_decl.source.value.as_str();
                  let imported_name = import_spec.imported.name();
                  if let Some(target_file) =
                    Self::resolve_imported_file(node_store, file_path, source)
                  {
                    if let Some(value) = Self::resolve_exported_symbol(
                      node_store,
                      target_file.as_str(),
                      imported_name.as_str(),
                      visited,
                    ) {
                      return Some(value);
                    }
                  }
                }
              }
            }
          }
        }
        Statement::VariableDeclaration(var_decl) => {
          for declarator in &var_decl.declarations {
            if let BindingPatternKind::BindingIdentifier(binding_ident) = &declarator.id.kind {
              if binding_ident.name == identifier {
                if let Some(init) = &declarator.init {
                  if let Some(value) =
                    Self::resolve_expression(node_store, file_path, init, visited)
                  {
                    return Some(value);
                  }
                }
              }
            }
          }
        }
        _ => {}
      }
    }

    Self::resolve_exported_symbol(node_store, file_path, identifier, visited)
  }

  fn resolve_exported_symbol(
    node_store: &NodeStore,
    file_path: &str,
    exported_name: &str,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    let source_text = std::fs::read_to_string(file_path).ok()?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).ok()?;
    let parser = Parser::new(&allocator, &source_text, source_type);
    let program = parser.parse().program;

    for stmt in &program.body {
      if let Statement::ExportNamedDeclaration(export_decl) = stmt {
        if let Some(declaration) = &export_decl.declaration {
          if let Declaration::VariableDeclaration(var_decl) = declaration {
            for declarator in &var_decl.declarations {
              if let BindingPatternKind::BindingIdentifier(binding_ident) = &declarator.id.kind {
                if binding_ident.name == exported_name {
                  if let Some(init) = &declarator.init {
                    if let Some(value) =
                      Self::resolve_expression(node_store, file_path, init, visited)
                    {
                      return Some(value);
                    }
                  }
                }
              }
            }
          }
        }

        for specifier in &export_decl.specifiers {
          if specifier.exported.name() != exported_name {
            continue;
          }

          let local_name = specifier.local.name();
          if let Some(source) = &export_decl.source {
            if let Some(target_file) =
              Self::resolve_imported_file(node_store, file_path, source.value.as_str())
            {
              if let Some(value) = Self::resolve_exported_symbol(
                node_store,
                target_file.as_str(),
                local_name.as_str(),
                visited,
              ) {
                return Some(value);
              }
            }
          } else if let Some(value) =
            Self::resolve_identifier_in_file(node_store, file_path, local_name.as_str(), visited)
          {
            return Some(value);
          }
        }
      }

      if exported_name == "default" {
        if let Statement::ExportDefaultDeclaration(default_decl) = stmt {
          if let ExportDefaultDeclarationKind::Identifier(ident) = &default_decl.declaration {
            if let Some(value) =
              Self::resolve_identifier_in_file(node_store, file_path, ident.name.as_str(), visited)
            {
              return Some(value);
            }
          }
        }
      }
    }

    None
  }

  fn resolve_expression(
    node_store: &NodeStore,
    file_path: &str,
    expr: &Expression,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    match expr {
      Expression::StringLiteral(s) => Some(s.value.to_string()),
      Expression::TemplateLiteral(tpl) => {
        if tpl.expressions.is_empty() && tpl.quasis.len() == 1 {
          return Some(tpl.quasis[0].value.raw.to_string());
        }
        None
      }
      Expression::Identifier(ident) => {
        Self::resolve_identifier_in_file(node_store, file_path, ident.name.as_str(), visited)
      }
      Expression::BinaryExpression(bin_expr) => {
        let left = Self::resolve_expression(node_store, file_path, &bin_expr.left, visited)?;
        let right = Self::resolve_expression(node_store, file_path, &bin_expr.right, visited)?;
        Some(format!("{}{}", left, right))
      }
      _ => None,
    }
  }

  fn resolve_imported_file(
    node_store: &NodeStore,
    file_path: &str,
    source: &str,
  ) -> Option<String> {
    if let Some(current_node) = node_store.get_node(file_path) {
      if let Some(importing_node) = current_node.get_importing_node(source) {
        return Some(importing_node.file_path.to_string());
      }
    }

    // Fallback: best-effort path resolution for rare cases where importing map is incomplete.
    let current_path = Path::new(file_path);
    let base = current_path.parent().unwrap_or_else(|| Path::new("."));
    let resolver = Resolver::new(Default::default());
    let resolved = resolver.resolve(base.to_str()?, source).ok()?;
    resolved.path().to_str().map(|s| s.to_string())
  }
}
