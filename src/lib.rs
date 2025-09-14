mod analyzer;
pub mod collector;
mod node;
mod walk_utils;

#[macro_use]
extern crate napi_derive;

use crate::analyzer::i18n_packages::I18nPackage;
use crate::node::node_store::NodeStore;
use analyzer::analyzer::Analyzer;
use collector::collector::Collector;
use std::collections::HashMap;
use log::info;

#[napi(object)]
pub struct Payload {
  pub tsconfig_path: String,
  pub entry_paths: Vec<String>,
  pub externals: Vec<String>,
  pub extend_i18n_packages: Option<Vec<I18nPackage>>,
}

/// This will follow the below flows to collect all the
/// I18n contents via passed entry file
/// 1. analyze all file references from the entry file
/// 2. find all import statements are includes the list below:
///    - import * from 'i18next'
///    - import * from 'react-i18next'
/// 3. find out the variable linked to the import statement
/// 4. recursively analyze the variable's references
/// 5. collect the first parameter of i18n function call
///    or bypass from another function wrapped by the i18n function
#[napi]
pub fn scan(payload: Payload) -> HashMap<String, Vec<String>> {
  // Initialize logger only once
  std::sync::Once::new().call_once(|| {
    env_logger::init();
  });
  
  if payload.entry_paths.len() < 1 {
    panic!("entry_paths is empty");
  }
  let node_store = NodeStore::default();

  let mut analyzer = Analyzer::new(
    node_store.clone(),
    payload.tsconfig_path,
    payload.externals.clone(),
  );

  analyzer.seed(
    &payload.entry_paths.get(0).unwrap(),
    payload.extend_i18n_packages,
  );

  payload.entry_paths.iter().for_each(|entry| {
    analyzer.analyze(entry.clone(), None);
  });

  info!(
    "[i18n-scanner-rs] found {} modules includes i18n",
    node_store.get_all_i18n_nodes().len()
  );

  let mut collector = Collector::new(node_store);

  collector.collect_keys();

  collector.i18n_namespaces
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
  use crate::analyzer::analyzer::Analyzer;
  use crate::collector::collector::Collector;
  use crate::node::node_store::NodeStore;
  use std::path::PathBuf;
  use std::rc::Rc;
  use crate::node::node::Node;
  use log::info;
  

  #[test]
  fn case_test() {
    let node_store = NodeStore::default();
    let base_path = "";
    let tsconfig_path = PathBuf::from(&base_path).join("tsconfig.json");
    let entry = PathBuf::from(&base_path).join("");

    let mut analyzer = Analyzer::new(
      node_store.clone(),
      tsconfig_path.to_str().unwrap().to_string(),
      vec![],
    );

    analyzer
      .seed(entry.to_str().unwrap(), None)
      .analyze(entry.to_str().unwrap().to_string(), None);

    // assert_eq!(node_store.get_i18n_exported_nodes().len(), 1);

    let mut collector = Collector::new(node_store);
    let keys = collector.collect_keys();

    info!("{:?}", keys.i18n_namespaces.get("default"));
  }
}
