use crate::analyzer::analyzer::Analyzer;
use crate::analyzer::i18n_packages::{I18nPackage, Member};
use crate::node::i18n_types::I18nType;
use crate::node::node_store::NodeStore;
use fs::canonicalize;
use std::{env, fs};

pub fn test_path(file_suffix: &str) -> String {
  let p = env::current_dir()
    .unwrap()
    .join("tests/fake-project/src/")
    .join(file_suffix);

  canonicalize(&p)
    .expect(format!("Failed to read {}", p.to_str().unwrap()).as_str())
    .display()
    .to_string()
}

pub fn analyze(entry: String, extend_packages: Option<Vec<I18nPackage>>) -> (Analyzer, NodeStore) {
  let node_store = NodeStore::default();

  let mut analyzer = Analyzer::new(node_store.clone(), test_path("../tsconfig.json"), vec![]);

  let source_path = test_path(entry.as_str());

  analyzer
    .seed(&source_path, extend_packages)
    .analyze(source_path, None);

  (analyzer, node_store)
}

pub fn make_extend_packages() -> Vec<I18nPackage> {
  vec![I18nPackage {
    package_path: test_path("WrapUseTranslationNs/hook.ts"),
    members: vec![Member {
      r#type: I18nType::Hook,
      name: "useFeTranslation".to_string(),
      ns: Some("namespace_3".into()),
    }],
  }]
}
