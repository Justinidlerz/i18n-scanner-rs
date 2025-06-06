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
        // tpl.
        // TODO: Handle template literal
        None
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
            AstKind::ImportSpecifier(_) => {
              // TODO: Handle key import cross file
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
}
