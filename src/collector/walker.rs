use crate::collector::post_collector::PostCollector;
use crate::node::node::Node;
use crate::walk_utils::WalkerUtils;
use oxc_ast::ast::{
  BindingPatternKind, CallExpression, Expression, ImportSpecifier, ObjectPropertyKind, PropertyKey,
};
use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::symbol::SymbolId;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Walker<'a> {
  pub node: Rc<Node>,
  pub semantic: &'a Semantic<'a>,
  pub i18n_namespaces: HashMap<String, Vec<String>>,
  pub post_collects: PostCollector,
  pub walk_utils: WalkerUtils<'a>,
}

impl<'a> Walker<'a> {
  pub fn new(node: Rc<Node>, semantic: &'a Semantic<'a>) -> Self {
    Self {
      node: node.clone(),
      semantic,
      i18n_namespaces: HashMap::new(),
      post_collects: PostCollector::new(),
      walk_utils: WalkerUtils::new(semantic, node.clone()),
    }
  }

  pub fn read_t(&mut self, symbol_id: SymbolId, namespace: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) else {
          return;
        };
        match node.kind() {
          AstKind::CallExpression(call) => {
            self.read_t_arguments(call, namespace.clone());
          }
          // TODO: handle bypass?
          _ => {}
        }
      })
  }

  pub fn read_object_member_t(&mut self, symbol_id: SymbolId, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(symbol_id)
      .for_each(|ref_item| {
        let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) else {
          return;
        };

        match node.kind() {
          // member call case
          // abc.t("abc")
          AstKind::MemberExpression(member) => {
            member.static_property_name().map(|prop| {
              if prop.eq("t") {
                if let Some(call_node) = self.semantic.nodes().parent_node(node.id()) {
                  if let AstKind::CallExpression(call) = call_node.kind() {
                    self.read_t_arguments(call, defined_ns.clone());
                  }
                }
              }
            });
          }
          // deconstruct case
          // const { t } = xyz;
          AstKind::VariableDeclaration(_var) => {
            // TODO: Deconstruct t from useTranslation case
          }
          _ => {}
        }
      })
  }

  pub fn read_hook(&mut self, s: &oxc_allocator::Box<ImportSpecifier>, defined_ns: Option<String>) {
    self
      .semantic
      .symbol_references(s.local.symbol_id.get().unwrap())
      .for_each(|ref_item| {
        let Some(node) = self.semantic.nodes().parent_node(ref_item.node_id()) else {
          return;
        };
        // only call useTranslation method but not assignment
        let Some(assign_node) = self.semantic.nodes().parent_node(node.id()) else {
          return;
        };

        // const xyz = useTranslation();
        let AstKind::VariableDeclarator(var) = assign_node.kind() else {
          return;
        };

        let namespace = match node.kind() {
          AstKind::CallExpression(call) => self.walk_utils.read_hook_namespace_argument(&call),
          _ => None,
        }
        .or_else(|| defined_ns.clone());

        match &var.id.kind {
          // const { t } = useTranslation();
          BindingPatternKind::ObjectPattern(obj) => obj.properties.iter().for_each(|prop| {
            if let PropertyKey::StaticIdentifier(key) = &prop.key {
              if key.name == "t" {
                if let BindingPatternKind::BindingIdentifier(ident) = &prop.value.kind {
                  self.read_t(ident.symbol_id(), namespace.clone());
                }
              }
            }
          }),
          // const trans = useTranslation();
          BindingPatternKind::BindingIdentifier(ident) => {
            self.read_object_member_t(ident.symbol_id(), namespace)
          }
          _ => {}
        }
      });
  }

  pub fn read_t_arguments(&mut self, call: &CallExpression, namespace: Option<String>) {
    let Some(arg) = call.arguments.get(0) else {
      return;
    };
    let key = arg
      .as_expression()
      .and_then(|expr| self.walk_utils.read_str_expression(&expr));

    let Some(key) = key else {
      return;
    };

    let ns = call
      .arguments
      .get(1)
      .and_then(|arg| arg.as_expression())
      .and_then(|expr| {
        // t("abc", { ns: "xyz" })
        if let Expression::ObjectExpression(obj) = expr {
          obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(obj_prop) = prop else {
              return None;
            };

            let PropertyKey::StaticIdentifier(prop_name) = &obj_prop.key else {
              return None;
            };

            if prop_name.name == "ns" {
              return self.walk_utils.read_str_expression(&obj_prop.value);
            }
            None
          })
        } else {
          None
        }
      })
      .or_else(|| namespace.and_then(|ns| Some(ns)))
      .unwrap_or_else(|| "default".to_string());

    self.add_key(&ns, key);
  }

  pub fn add_key(&mut self, namespace: &str, key: String) {
    self
      .i18n_namespaces
      .entry(namespace.to_string())
      .or_insert_with(Vec::new)
      .push(key);
  }
}
