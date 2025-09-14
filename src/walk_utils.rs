use crate::node::node::Node;
use oxc_ast::ast::{CallExpression, Expression};
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
      },
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
              // Try to resolve the actual value from the imported module
              if let Some(imported_value) = self.resolve_imported_value(&imported_name) {
                return Some(imported_value);
              }
              // Fallback to the imported name
              return Some(imported_name.to_string());
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
        if bin_expr.operator == oxc_ast::ast::BinaryOperator::Addition {
          // Handle string concatenation
          if let (Some(left), Some(right)) = (
            self.read_str_expression(&bin_expr.left),
            self.read_str_expression(&bin_expr.right)
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

    arg
      .as_expression()
      .and_then(|expr| self.read_str_expression(expr))
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
              if let oxc_ast::ast::Statement::ExpressionStatement(expr_stmt) = stmt {
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

  pub fn resolve_imported_value(&self, _imported_name: &str) -> Option<String> {
    // Try to resolve cross-file imports by looking at the current node's imports
    // This is a simplified implementation that works with the existing infrastructure
    
    // First, we need to find which import declaration this imported_name comes from
    // Then get the importing node and look for the exported constant
    
    // For now, we'll try a simple approach: look through all importing nodes
    // and see if any of them export this constant
    
    // This is still a TODO that requires more complex implementation
    // involving AST parsing of imported files to find exported constants
    None
  }
}
