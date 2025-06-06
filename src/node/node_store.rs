use crate::node::node::{Node, NodePath};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type NodeRecord = HashMap<NodePath, Rc<Node>>;

#[derive(Default, Debug)]
pub struct NodeStore(Rc<RefCell<NodeRecord>>);

impl NodeStore {
  pub fn new(node_record: NodeRecord) -> Self {
    Self(Rc::new(RefCell::new(node_record)))
  }
  pub fn get_by_node_path(&self, node_path: &NodePath) -> Option<Rc<Node>> {
    self.0.borrow().get(node_path).map(|node| Rc::clone(node))
  }
  pub fn get_node(&self, file_path: &str) -> Option<Rc<Node>> {
    self
      .0
      .borrow()
      .get(&file_path.to_string())
      .map(|node| Rc::clone(node))
  }

  pub fn get_path_and_node(&self, file_path: &str) -> Option<(NodePath, Rc<Node>)> {
    self
      .0
      .borrow()
      .get_key_value(&file_path.to_string())
      .map(|(k, v)| (k.clone(), Rc::clone(v)))
  }

  pub fn insert_node(&self, file_path: Rc<String>, node: Rc<Node>) {
    self.0.borrow_mut().insert(file_path, node);
  }
  pub fn clone(&self) -> Self {
    Self(Rc::clone(&self.0))
  }

  pub fn get_all_nodes(&self) -> NodeRecord {
    self.0.borrow().clone()
  }

  pub fn get_i18n_exported_nodes(&self) -> NodeRecord {
    self
      .0
      .borrow()
      .iter()
      .filter(|(_, node)| node.has_exported_i18n_methods())
      .map(|(k, v)| (k.clone(), Rc::clone(v)))
      .collect()
  }

  pub fn get_all_i18n_nodes(&self) -> NodeRecord {
    self
      .0
      .borrow()
      .iter()
      .filter_map(|(k, node)| {
        if node.has_i18n_source_imported() {
          Some((k.clone(), Rc::clone(node)))
        } else {
          None
        }
      })
      .collect()
  }
}
