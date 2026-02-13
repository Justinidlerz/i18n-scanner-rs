use crate::analyzer::resolver::create_resolver;
use crate::analyzer::walker::Walker;
use crate::node::node::Node;
use crate::node::node_store::NodeStore;
use oxc_allocator::Allocator;
use oxc_ast_visit::walk;
use oxc_parser::Parser;
use oxc_resolver::Resolver;
use oxc_semantic::SemanticBuilder;
use regex::Regex;
use std::fs;
use std::rc::Rc;

pub struct Analyzer {
  pub node_store: NodeStore,
  allocator: Allocator,
  pub resolver: Rc<Resolver>,
  script_tester: Regex,
  externals: Rc<Vec<Regex>>,
}

impl Analyzer {
  pub fn new(all_nodes: NodeStore, tsconfig_path: String, externals: Vec<String>) -> Self {
    Self {
      node_store: all_nodes,
      externals: Rc::new(
        externals
          .iter()
          .map(|pkg_name| Regex::new(&format!(r"^{}((!?\/).*)?$", pkg_name)).unwrap())
          .collect(),
      ),
      allocator: Allocator::default(),
      resolver: Rc::new(create_resolver(tsconfig_path)),
      script_tester: Regex::new(r"^.+\.(ts|tsx|js|jsx)$").unwrap(),
    }
  }

  pub fn analyze(
    &mut self,
    file_path: String,
    imports_path: Option<Rc<String>>,
  ) -> Option<Rc<Node>> {
    if let Some(existing_node) = self.node_store.get_node(&file_path) {
      if let Some(path) = imports_path {
        existing_node.insert_imports(path.clone());
      }
      return Some(existing_node);
    }
    if !self.script_tester.is_match(&file_path) {
      return None;
    }

    let file_path_ref = Rc::new(file_path);
    let node = Rc::new(Node::new(file_path_ref.clone(), self.node_store.clone()));
    let i18n_nodes = NodeStore::new(self.node_store.get_i18n_exported_nodes());

    if let Some(path) = imports_path {
      node.insert_imports(path.clone());
    }

    let source_text = match fs::read_to_string(file_path_ref.as_ref()) {
      Ok(source) => source,
      Err(err) => {
        log::warn!(
          "[i18n-scanner-rs] Unable to read file at {}: {err}",
          file_path_ref
        );
        return None;
      }
    };

    self.node_store.insert_node(file_path_ref, node.clone());

    let parser = Parser::new(&self.allocator, &source_text, node.source_type);
    let program = parser.parse().program;
    let semantic = SemanticBuilder::new().build(&program);

    let mut walker = Walker::new(
      Rc::clone(&self.resolver),
      node.clone(),
      i18n_nodes,
      &semantic.semantic,
      self.externals.clone(),
    );

    walk::walk_program(&mut walker, &program);

    for (source, path) in walker.get_importing_collection().iter() {
      if let Some(new_node) = self.analyze(path.to_string(), Some(node.file_path.clone())) {
        node.insert_importing(source.to_string(), new_node.file_path.clone());

        if new_node.has_exported_i18n_methods() {
          node.mark_has_i18n_source_imported();
        }
      }
    }

    Some(node)
  }
}

#[cfg(test)]
mod tests {
  use crate::analyzer::test_utils::{
    analyze, analyze_with_options, make_custom_i18n_package, make_extend_packages,
  };

  #[test]
  fn make_seed() {
    let (_, node_store) = analyze("index.tsx".into(), None);

    assert_eq!(node_store.get_i18n_exported_nodes().len(), 5);
  }

  #[test]
  fn node_include_i18n_import() {
    let (_, node_store) = analyze("index.tsx".into(), None);

    assert_eq!(node_store.get_all_i18n_nodes().len(), 28);
  }

  #[test]
  fn extended_i18n_package() {
    let pkgs = make_extend_packages();

    let (_, node_store) = analyze("index.tsx".into(), Some(pkgs));

    assert_eq!(node_store.get_i18n_exported_nodes().len(), 5);
  }

  #[test]
  fn filter_externals() {
    // let all_nodes = NodeStore::default();

    // vec![
    //   "@i18n-ecom-seller/[^-]+-exp".into(),
    //   "mf_.*".into(),
    //   "@dynokit/.*".into(),
    // ],
  }

  #[test]
  fn extend_custom_package_with_default_members() {
    let packages = make_custom_i18n_package();
    let (_, node_store) = analyze("custom-i18n/index.tsx".into(), Some(packages));

    let exported_nodes = node_store.get_i18n_exported_nodes();
    let custom_node = exported_nodes
      .iter()
      .find(|(path, _)| path.ends_with("tests/custom-i18n/index.ts"))
      .map(|(_, node)| node.clone());

    let Some(node) = custom_node else {
      panic!("custom i18n package was not seeded");
    };

    let members = node.get_exporting_i18n_members();

    for member_name in [
      "t",
      "useTranslation",
      "Trans",
      "Translation",
      "withTranslation",
      "i18n",
    ] {
      assert!(
        members.contains_key(member_name),
        "missing member {member_name}"
      );
    }
  }

  #[test]
  fn externals_keep_preset_packages_available() {
    let (_, node_store) = analyze_with_options(
      "index.tsx".into(),
      None,
      vec!["i18next".into(), "react-i18next".into()],
    );

    assert!(node_store.get_all_i18n_nodes().len() > 0);
  }

  #[test]
  fn externals_keep_custom_package_available() {
    let packages = make_custom_i18n_package();
    let (_, node_store) = analyze_with_options(
      "custom-i18n/index.tsx".into(),
      Some(packages),
      vec![
        "@custom/i18n".into(),
        "i18next".into(),
        "react-i18next".into(),
      ],
    );

    assert!(node_store.get_all_i18n_nodes().len() > 0);
    assert!(node_store
      .get_i18n_exported_nodes()
      .keys()
      .any(|path| path.ends_with("tests/custom-i18n/index.ts")));
  }
}
