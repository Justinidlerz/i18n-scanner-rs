use std::rc::Weak;

pub struct KeyRef {
  member: String,
  node: Weak<String>,
}

pub enum KeyPath {
  String(String),
  LinkedKey(KeyRef),
}

impl KeyPath {
  pub fn from_string(s: String) -> Self {
    Self::String(s)
  }
  pub fn from_node(node: Weak<String>, member: String) -> Self {
    Self::LinkedKey(KeyRef { member, node })
  }
}

pub struct PostCollector(Vec<(Vec<KeyPath>, Vec<KeyPath>)>);

impl PostCollector {
  pub fn new() -> Self {
    Self(vec![])
  }

  pub fn add(&mut self, key_path: Vec<KeyPath>, value_path: Vec<KeyPath>) {
    self.0.push((key_path, value_path));
  }
}
