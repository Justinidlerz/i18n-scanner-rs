use crate::node::node::Node;
use log::debug;
use oxc_ast::ast::{
  BinaryOperator, BindingPatternKind, CallExpression, Declaration, Expression, ObjectPropertyKind,
  PropertyKey, SourceType, Statement,
};
use oxc_ast::AstKind;
use oxc_semantic::{AstNode, Semantic};
use oxc_syntax::reference::ReferenceId;
use std::rc::Rc;

pub struct WalkerUtils<'a> {
  pub semantic: &'a Semantic<'a>,
  pub node: Rc<Node>,
}

impl<'a> WalkerUtils<'a> {
  pub fn new(semantic: &'a Semantic<'a>, node: Rc<Node>) -> Self {
    Self { semantic, node }
  }

  pub fn read_str_expression(&self, expr: &Expression) -> Option<String> {
    match expr {
      Expression::StringLiteral(s) => Some(s.value.to_string()),
      Expression::TemplateLiteral(tpl) => {
        // Handle template literal by concatenating quasis and expressions
        if tpl.quasis.len() == 1 && tpl.expressions.is_empty() {
          // Simple template literal with no expressions: `"hello"`
          Some(tpl.quasis[0].value.raw.to_string())
        } else {
          // Template literal with expressions: `"hello" + world`
          let mut result = String::new();
          for (i, quasi) in tpl.quasis.iter().enumerate() {
            result.push_str(&quasi.value.raw);
            if i < tpl.expressions.len() {
              // Try to resolve the expression to a string
              if let Some(expr_str) = self.read_str_expression(&tpl.expressions[i]) {
                result.push_str(&expr_str);
              } else {
                // If we can't resolve the expression, return None
                return None;
              }
            }
          }
          Some(result)
        }
      }
      // value ref
      Expression::Identifier(ident) => {
        let get_node_content = |node: &AstNode| {
          match node.kind() {
            AstKind::VariableDeclarator(var) => {
              if let Some(init) = &var.init {
                return self.read_str_expression(init);
              }
              // TODO: Handle define and after assignment
              //  let key;
              //  key = "abc";
            }
            AstKind::ImportSpecifier(import_spec) => {
              // Handle key import cross file
              let imported_name = import_spec.imported.name();
              debug!("Found import specifier for: {}", imported_name);

              // Find the import declaration to get the source
              if let Some(parent_node) = self.semantic.nodes().parent_node(node.id()) {
                if let AstKind::ImportDeclaration(import_decl) = parent_node.kind() {
                  let source = import_decl.source.value.as_str();
                  debug!("Import source: {}", source);

                  // Try to resolve the actual value from the imported module
                  if let Some(imported_value) =
                    self.resolve_imported_value_from_source(source, &imported_name)
                  {
                    return Some(imported_value);
                  }
                }
              }

              // Keep unresolved imports as None so post-collection can resolve
              // cross-file alias chains instead of collecting identifier names.
              return None;
            }
            _ => {}
          }
          None
        };
        if let Some(node) = self.get_var_defined_node(ident.reference_id()) {
          if let Some(content) = get_node_content(node) {
            return Some(content);
          }
        }
        if let Some(node) = self.get_var_defined_parent_node(ident.reference_id()) {
          if let Some(content) = get_node_content(node) {
            return Some(content);
          }
        }
        None
      }
      // Handle binary expressions like 'prefix' + '_' + variable
      Expression::BinaryExpression(bin_expr) => {
        if bin_expr.operator == BinaryOperator::Addition {
          // Handle string concatenation
          if let (Some(left), Some(right)) = (
            self.read_str_expression(&bin_expr.left),
            self.read_str_expression(&bin_expr.right),
          ) {
            return Some(format!("{}{}", left, right));
          }
        }
        None
      }
      // Handle array operations for dynamic keys
      Expression::CallExpression(call) => {
        if let Some(callee) = call.callee.as_member_expression() {
          if let Some(prop_name) = callee.static_property_name() {
            if prop_name == "map" {
              // Handle array.map() operations for dynamic key generation
              return self.read_dynamic_keys_from_map(call);
            }
          }
        }
        None
      }
      _ => None,
    }
  }

  pub fn read_hook_namespace_argument(&self, call: &CallExpression) -> Option<String> {
    let Some(arg) = call.arguments.get(0) else {
      return None;
    };

    arg.as_expression().and_then(|expr| match expr {
      // useTranslation('namespace')
      Expression::StringLiteral(_) | Expression::TemplateLiteral(_) | Expression::Identifier(_) => {
        self.read_str_expression(expr)
      }
      // useTranslation(['namespaceA', 'namespaceB'])
      Expression::ArrayExpression(array_expr) => self.read_first_namespace_from_array(array_expr),
      // useTranslation({ ns: 'namespace' })
      Expression::ObjectExpression(obj_expr) => obj_expr.properties.iter().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(object_prop) = prop else {
          return None;
        };

        let PropertyKey::StaticIdentifier(key) = &object_prop.key else {
          return None;
        };

        if key.name != "ns" {
          return None;
        }

        // Prefer centralized string resolution to keep namespace parsing consistent.
        self.read_str_expression(&object_prop.value)
      }),
      _ => None,
    })
  }

  fn read_first_namespace_from_array(
    &self,
    array_expr: &oxc_ast::ast::ArrayExpression,
  ) -> Option<String> {
    let first = array_expr.elements.first()?;
    let expr = first.as_expression()?;

    // react-i18next uses the first namespace as default for t().
    self.read_str_expression(expr)
  }

  pub fn get_var_defined_node(&self, ref_id: ReferenceId) -> Option<&AstNode> {
    self
      .semantic
      .scoping()
      .get_reference(ref_id)
      .symbol_id()
      .and_then(|symbol_id| Some(self.semantic.symbol_declaration(symbol_id)))
  }

  pub fn get_var_defined_parent_node(&self, ref_id: ReferenceId) -> Option<&AstNode> {
    self
      .get_var_defined_node(ref_id)
      .and_then(|node| self.semantic.nodes().parent_node(node.id()))
  }

  pub fn read_dynamic_keys_from_map(&self, call: &CallExpression) -> Option<String> {
    // For dynamic key generation like array.map((v) => t(prefix + '_' + v))
    // We need to:
    // 1. Find the array being mapped over
    // 2. Resolve the array elements
    // 3. Extract the key pattern from the arrow function
    // 4. Generate keys by substituting each array element
    //
    // This is a complex feature that requires:
    // - Array value resolution
    // - Parameter substitution in expressions
    // - Multiple key generation
    //
    // TODO: Implement proper dynamic key resolution without hardcoding

    if let Some(arg) = call.arguments.get(0) {
      if let Some(expr) = arg.as_expression() {
        match expr {
          Expression::ArrowFunctionExpression(arrow) => {
            let body = &arrow.body;
            // FunctionBody is a struct with statements field
            for stmt in &body.statements {
              if let Statement::ExpressionStatement(expr_stmt) = stmt {
                let expr = &expr_stmt.expression;
                return self.read_dynamic_key_expression(expr);
              }
            }
          }
          _ => {}
        }
      }
    }
    None
  }

  pub fn read_dynamic_key_expression(&self, expr: &Expression) -> Option<String> {
    match expr {
      Expression::CallExpression(call) => {
        // This is a t() call within the map function
        if let Some(arg) = call.arguments.get(0) {
          if let Some(expr) = arg.as_expression() {
            return self.read_str_expression(expr);
          }
        }
        None
      }
      _ => None,
    }
  }

  pub fn resolve_imported_value(&self, imported_name: &str) -> Option<String> {
    // Legacy method - kept for compatibility
    debug!("Trying to resolve imported value: {}", imported_name);

    // For now, let's try to resolve by checking if there's an importing node for this specifier
    if let Some(importing_node) = self.node.get_importing_node(imported_name) {
      debug!("Found importing node: {}", importing_node.file_path);
      let result = self.resolve_exported_constant_value(&importing_node.file_path, imported_name);
      debug!("Resolved value: {:?}", result);
      return result;
    } else {
      debug!("No importing node found for: {}", imported_name);
    }

    None
  }

  pub fn resolve_imported_value_from_source(
    &self,
    source: &str,
    imported_name: &str,
  ) -> Option<String> {
    // Resolve imported value given the import source and the imported member name
    debug!(
      "Trying to resolve imported value '{}' from source '{}'",
      imported_name, source
    );

    // Get the importing node using the source path
    if let Some(importing_node) = self.node.get_importing_node(source) {
      debug!("Found importing node: {}", importing_node.file_path);
      let result = self.resolve_exported_constant_value(&importing_node.file_path, imported_name);
      debug!("Resolved value: {:?}", result);
      return result;
    } else {
      debug!("No importing node found for source: {}", source);
    }

    None
  }

  fn resolve_exported_constant_value(
    &self,
    file_path: &str,
    constant_name: &str,
  ) -> Option<String> {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use std::fs;

    // Read the file content
    let source_text = fs::read_to_string(file_path).ok()?;

    // Parse the file
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file_path).unwrap_or_default();
    let parser = Parser::new(&allocator, &source_text, source_type);
    let program = parser.parse().program;

    // Look for export const declarations
    for stmt in &program.body {
      match stmt {
        Statement::ExportNamedDeclaration(export_decl) => {
          if let Some(decl) = &export_decl.declaration {
            if let Declaration::VariableDeclaration(var_decl) = decl {
              for declarator in &var_decl.declarations {
                if let BindingPatternKind::BindingIdentifier(ident) = &declarator.id.kind {
                  if ident.name == constant_name {
                    // Found the constant, try to extract its value
                    if let Some(init) = &declarator.init {
                      return self.extract_constant_value(init);
                    }
                  }
                }
              }
            }
          }
        }
        _ => {}
      }
    }

    None
  }

  fn extract_constant_value(&self, expr: &Expression) -> Option<String> {
    match expr {
      Expression::StringLiteral(s) => Some(s.value.to_string()),
      Expression::TemplateLiteral(tpl) => {
        // Handle simple template literals
        if tpl.quasis.len() == 1 && tpl.expressions.is_empty() {
          Some(tpl.quasis[0].value.raw.to_string())
        } else {
          None // Complex template literals not supported yet
        }
      }
      _ => None,
    }
  }
}
