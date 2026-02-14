use crate::node::node_store::NodeStore;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
  BindingPatternKind, Declaration, ExportDefaultDeclarationKind, Expression,
  ImportDeclarationSpecifier, SourceType, Statement,
};
use oxc_parser::Parser;
use std::collections::{HashMap, HashSet};

pub struct PendingIdentifierKey {
  pub node_path: String,
  pub namespace: String,
  pub identifier: String,
}

#[derive(Clone, Debug)]
enum ValueExpr {
  String(String),
  Identifier(String),
  Binary(Box<ValueExpr>, Box<ValueExpr>),
}

#[derive(Default)]
struct ModuleInfo {
  locals: HashMap<String, ValueExpr>,
  imports: HashMap<String, (String, String)>,
  exports: HashMap<String, String>,
  export_values: HashMap<String, ValueExpr>,
  export_default_ident: Option<String>,
}

struct ResolveContext<'a> {
  node_store: &'a NodeStore,
  module_cache: HashMap<String, ModuleInfo>,
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
    let mut ctx = ResolveContext {
      node_store,
      module_cache: HashMap::new(),
    };

    for pending in &self.pending_identifier_keys {
      let mut visited = HashSet::new();
      if let Some(value) = Self::resolve_identifier_in_file(
        &mut ctx,
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
    ctx: &mut ResolveContext,
    file_path: &str,
    identifier: &str,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    let visit_key = format!("{}::{}", file_path, identifier);
    if !visited.insert(visit_key) {
      // Guard against cycles across aliases/import-export chains.
      return None;
    }

    let module = Self::load_module_info(ctx, file_path)?;

    // Resolve local symbol first because `t(KEY)` points to local scope binding.
    if let Some(local_expr) = module.locals.get(identifier).cloned() {
      return Self::resolve_value_expr(ctx, file_path, &local_expr, visited);
    }

    // Resolve imported binding by following NodeStore import graph directly.
    if let Some((source, imported_name)) = module.imports.get(identifier).cloned() {
      if let Some(target_file) = Self::resolve_imported_file(ctx, file_path, &source) {
        return Self::resolve_exported_symbol(ctx, &target_file, &imported_name, visited);
      }
    }

    Self::resolve_exported_symbol(ctx, file_path, identifier, visited)
  }

  fn resolve_exported_symbol(
    ctx: &mut ResolveContext,
    file_path: &str,
    exported_name: &str,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    let module = Self::load_module_info(ctx, file_path)?;

    if let Some(export_expr) = module.export_values.get(exported_name).cloned() {
      return Self::resolve_value_expr(ctx, file_path, &export_expr, visited);
    }

    if let Some(local_name) = module.exports.get(exported_name).cloned() {
      return Self::resolve_identifier_in_file(ctx, file_path, &local_name, visited);
    }

    if exported_name == "default" {
      if let Some(default_ident) = module.export_default_ident.clone() {
        return Self::resolve_identifier_in_file(ctx, file_path, &default_ident, visited);
      }
    }

    None
  }

  fn resolve_value_expr(
    ctx: &mut ResolveContext,
    file_path: &str,
    expr: &ValueExpr,
    visited: &mut HashSet<String>,
  ) -> Option<String> {
    match expr {
      ValueExpr::String(s) => Some(s.clone()),
      ValueExpr::Identifier(ident) => {
        Self::resolve_identifier_in_file(ctx, file_path, ident.as_str(), visited)
      }
      ValueExpr::Binary(left, right) => {
        let left_value = Self::resolve_value_expr(ctx, file_path, left, visited)?;
        let right_value = Self::resolve_value_expr(ctx, file_path, right, visited)?;
        Some(format!("{}{}", left_value, right_value))
      }
    }
  }

  fn load_module_info<'a>(ctx: &'a mut ResolveContext, file_path: &str) -> Option<&'a ModuleInfo> {
    if !ctx.module_cache.contains_key(file_path) {
      let source_text = std::fs::read_to_string(file_path).ok()?;
      let allocator = Allocator::default();
      let source_type = SourceType::from_path(file_path).ok()?;
      let parser = Parser::new(&allocator, &source_text, source_type);
      let program = parser.parse().program;

      let mut module = ModuleInfo::default();

      for stmt in &program.body {
        match stmt {
          Statement::ImportDeclaration(import_decl) => {
            if let Some(specifiers) = &import_decl.specifiers {
              for specifier in specifiers {
                if let ImportDeclarationSpecifier::ImportSpecifier(import_spec) = specifier {
                  module.imports.insert(
                    import_spec.local.name.to_string(),
                    (
                      import_decl.source.value.to_string(),
                      import_spec.imported.name().to_string(),
                    ),
                  );
                }
              }
            }
          }
          Statement::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
              if let BindingPatternKind::BindingIdentifier(binding_ident) = &declarator.id.kind {
                if let Some(init) = &declarator.init {
                  if let Some(expr) = Self::extract_value_expr(init) {
                    module.locals.insert(binding_ident.name.to_string(), expr);
                  }
                }
              }
            }
          }
          Statement::ExportNamedDeclaration(export_decl) => {
            if let Some(declaration) = &export_decl.declaration {
              if let Declaration::VariableDeclaration(var_decl) = declaration {
                for declarator in &var_decl.declarations {
                  if let BindingPatternKind::BindingIdentifier(binding_ident) = &declarator.id.kind
                  {
                    if let Some(init) = &declarator.init {
                      if let Some(expr) = Self::extract_value_expr(init) {
                        module
                          .export_values
                          .insert(binding_ident.name.to_string(), expr.clone());
                        module.locals.insert(binding_ident.name.to_string(), expr);
                      }
                    }
                  }
                }
              }
            }

            for specifier in &export_decl.specifiers {
              module.exports.insert(
                specifier.exported.name().to_string(),
                specifier.local.name().to_string(),
              );
            }
          }
          Statement::ExportDefaultDeclaration(default_decl) => {
            if let ExportDefaultDeclarationKind::Identifier(ident) = &default_decl.declaration {
              module.export_default_ident = Some(ident.name.to_string());
            }
          }
          _ => {}
        }
      }

      ctx.module_cache.insert(file_path.to_string(), module);
    }

    ctx.module_cache.get(file_path)
  }

  fn extract_value_expr(expr: &Expression) -> Option<ValueExpr> {
    match expr {
      Expression::StringLiteral(s) => Some(ValueExpr::String(s.value.to_string())),
      Expression::TemplateLiteral(tpl) => {
        if tpl.expressions.is_empty() && tpl.quasis.len() == 1 {
          Some(ValueExpr::String(tpl.quasis[0].value.raw.to_string()))
        } else {
          None
        }
      }
      Expression::Identifier(ident) => Some(ValueExpr::Identifier(ident.name.to_string())),
      Expression::BinaryExpression(bin_expr) => {
        let left = Self::extract_value_expr(&bin_expr.left)?;
        let right = Self::extract_value_expr(&bin_expr.right)?;
        Some(ValueExpr::Binary(Box::new(left), Box::new(right)))
      }
      _ => None,
    }
  }

  fn resolve_imported_file(ctx: &ResolveContext, file_path: &str, source: &str) -> Option<String> {
    // Use analyzer-built NodeStore graph as the single source of truth.
    let current_node = ctx.node_store.get_node(file_path)?;
    let target_node = current_node.get_importing_node(source)?;
    Some(target_node.file_path.to_string())
  }
}
