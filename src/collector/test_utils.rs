use crate::analyzer::analyzer::Analyzer;
use crate::analyzer::i18n_packages::I18nPackage;
use crate::analyzer::test_utils::analyze;
use crate::collector::collector::Collector;
use log::debug;

#[macro_export]
macro_rules! key_match {
  ($name:ident, $entry:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let (_, collector) = collect($entry, None);
      let mut keys = collector.get_keys("default".into());
      let mut res = $expected.clone();
      res.sort();
      keys.sort();
      assert_eq!(keys, res);
    }
  };
  ($name:ident, $entry:expr, $ns:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let (_, collector) = collect($entry, None);
      let mut keys = collector.get_keys($ns);
      let mut res = $expected.clone();
      res.sort();
      keys.sort();
      assert_eq!(keys, res);
    }
  };
  ($name:ident, $entry:expr, $ns:expr, $extend_packages:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let (_, collector) = collect($entry, Some($extend_packages));
      let mut keys = collector.get_keys($ns);
      let mut res = $expected.clone();
      res.sort();
      keys.sort();
      assert_eq!(keys, res);
    }
  };
}

pub fn collect(entry: String, extend_packages: Option<Vec<I18nPackage>>) -> (Analyzer, Collector) {
  // Initialize logger for tests - use try_init to avoid panic if already initialized
  let _ = env_logger::try_init();

  let (analyzer, node_store) = analyze(entry, extend_packages);

  let with_i18n_nodes = node_store.get_all_i18n_nodes();

  debug!("with_i18n_nodes: {:?}", with_i18n_nodes.len());

  let mut collector = Collector::new(node_store.clone());

  collector.collect_keys();

  (analyzer, collector)
}
