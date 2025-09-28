use crate::node::i18n_types::I18nMember;
use crate::node::node_store::NodeStore;
use oxc_ast::ast::SourceType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type NodePath = Rc<String>;

#[derive(Debug)]
pub struct Node {
  pub file_path: NodePath,
  pub source_type: SourceType,
  node_store: NodeStore,
  // Imports this node's parent
  imports: RefCell<Vec<NodePath>>,
  // Import cannot judge what kind of import is
  // only collect the importing members
  // for referencing some cross file variables import
  // { "./xyz": node_path }
  importing: RefCell<HashMap<String, NodePath>>,
  // Export can judge what kind of export is but except reexports
  // but analyzer will update the export_members of imports
  // { "useFeTranslate": Trans, "xyz": Trans }
  exporting_members: RefCell<HashMap<String, Option<I18nMember>>>,
  // flag the node has exported the i18n translator
  // that means we need to tell the import node that we have exported the i18n translator
  has_exported_i18n_methods: RefCell<bool>,
  // flag the node has imported the i18n translator
  // that means we need to collect the i18n keys on this file node
  has_i18n_source_imported: RefCell<bool>,
}

impl Node {
  pub fn new(file_path: Rc<String>, node_store: NodeStore) -> Self {
    let source_type = SourceType::from_path(&file_path.as_str()).unwrap();

    Self {
      node_store,
      source_type,
      file_path,
      imports: RefCell::new(vec![]),
      importing: RefCell::new(HashMap::new()),
      exporting_members: RefCell::new(HashMap::new()),
      has_exported_i18n_methods: RefCell::new(false),
      has_i18n_source_imported: RefCell::new(false),
    }
  }

  pub fn mark_has_i18n_source_imported(&self) {
    *self.has_i18n_source_imported.borrow_mut() = true;
  }

  pub fn has_i18n_source_imported(&self) -> bool {
    *self.has_i18n_source_imported.borrow()
  }

  pub fn has_exported_i18n_methods(&self) -> bool {
    *self.has_exported_i18n_methods.borrow()
  }

  pub fn insert_imports(&self, parent_path: Rc<String>) {
    self.imports.borrow_mut().push(parent_path);
  }

  pub fn insert_exporting(&self, member: String, i18n_member: Option<I18nMember>) {
    if i18n_member.is_some() {
      *self.has_exported_i18n_methods.borrow_mut() = true;
    }

    self
      .exporting_members
      .borrow_mut()
      .insert(member, i18n_member);
  }

  pub fn try_insert_importing(&self, specifier: String, source_path: String) -> Result<(), ()> {
    let source = self.node_store.get_path_and_node(&source_path);
    let Some((path, node)) = source else {
      return Err(());
    };

    node.insert_imports(self.file_path.clone());

    let mut importing = self.importing.borrow_mut();

    importing.insert(specifier, path);

    drop(importing);
    Ok(())
  }

  pub fn insert_importing(&self, specifier: String, source_path: Rc<String>) {
    let mut importing = self.importing.borrow_mut();

    importing.insert(specifier, source_path);

    drop(importing);
  }

  pub fn get_exporting_members(&self) -> HashMap<String, Option<I18nMember>> {
    self.exporting_members.borrow().clone()
  }

  pub fn get_exporting_i18n_members(&self) -> HashMap<String, I18nMember> {
    self
      .exporting_members
      .borrow()
      .iter()
      .filter_map(|(name, translation_method)| {
        if translation_method.is_some() {
          return Some((name.to_string(), translation_method.clone().unwrap()));
        }
        None
      })
      .collect()
  }

  pub fn get_importing_node(&self, specifier: &str) -> Option<Rc<Node>> {
    if let Some(node_path) = self.importing.borrow().get(specifier) {
      return self.node_store.get_by_node_path(node_path);
    }
    None
  }
}
