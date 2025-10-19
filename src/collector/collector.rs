use crate::collector::walker::Walker;
use crate::node::node_store::NodeStore;
use oxc_allocator::Allocator;
use oxc_ast_visit::walk;
use oxc_minifier::{CompressOptions, MangleOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use std::collections::HashMap;
use std::fs;

pub struct Collector {
  node_store: NodeStore,
  allocator: Allocator,
  pub i18n_namespaces: HashMap<String, Vec<String>>,
}

impl Collector {
  pub fn new(nodes: NodeStore) -> Self {
    Self {
      node_store: nodes,
      allocator: Allocator::default(),
      i18n_namespaces: HashMap::new(),
    }
  }

  pub fn collect_keys(&mut self) -> &mut Self {
    let i18n_nodes = self.node_store.get_all_i18n_nodes();

    // let post_collects =
    for (_, node) in i18n_nodes.iter() {
      let source_text = fs::read_to_string(&node.file_path.as_str()).unwrap();
      let parser = Parser::new(&self.allocator, &source_text, node.source_type);
      let mut program = parser.parse().program;

      Minifier::new(MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress: Some(CompressOptions::safest()),
      })
      .build(&self.allocator, &mut program);

      let semantic = SemanticBuilder::new().build(&program);
      let mut walker = Walker::new(node.clone(), &semantic.semantic);

      walk::walk_program(&mut walker, &program);

      walker.i18n_namespaces.iter().for_each(|(namespace, keys)| {
        self
          .i18n_namespaces
          .entry(namespace.to_string())
          .or_default()
          .extend(keys.iter().cloned());
      })
    }
    self
  }

  pub fn get_keys(&self, namespace: &str) -> Vec<String> {
    let default = Vec::<String>::new();

    self
      .i18n_namespaces
      .get(namespace)
      .unwrap_or(&default)
      .clone()
  }
}

#[cfg(test)]
mod tests {
  use crate::analyzer::i18n_packages::{I18nPackage, Member};
  use crate::analyzer::test_utils::make_custom_i18n_package;
  use crate::collector::test_utils::{collect, collect_with_options};
  use crate::key_match;
  use crate::node::i18n_types::I18nType;

  #[test]
  fn full_collect() {
    let (_, collector) = collect("index.tsx".into(), None);

    assert_eq!(collector.i18n_namespaces.len(), 5);

    println!("default {:?}", collector.get_keys("default"));

    assert_eq!(collector.get_keys("default").len(), 16);
    assert_eq!(collector.get_keys("namespace_1").len(), 2);
    assert_eq!(collector.get_keys("namespace_2").len(), 1);
    assert_eq!(collector.get_keys("namespace_3").len(), 2);
    assert_eq!(collector.get_keys("namespace_4").len(), 1);
  }

  key_match!(
    nothing_about_i18n,
    "NothingAboutI18n.tsx".into(),
    Vec::<String>::new()
  );
  key_match!(
    i18n_instance_init_only,
    "i18nInstanceInitOnly.tsx".into(),
    Vec::<String>::new()
  );
  key_match!(
    hook_with_namespace,
    "HookWithNamespace.tsx".into(),
    "namespace_1".into(),
    vec!["HOOK_WITH_NAMESPACE"]
  );
  key_match!(
    t_with_namespace,
    "TWithNamespace.tsx".into(),
    "namespace_1".into(),
    vec!["T_WITH_NAMESPACE"]
  );
  key_match!(
    namespace_override,
    "NamespaceOverride.tsx".into(),
    "namespace_2".into(),
    vec!["NAMESPACE_OVERRIDE"]
  );
  key_match!(global_t, "globalT.ts".into(), vec!["GLOBAL_T"]);
  key_match!(rename_both, "RenameBoth.tsx".into(), vec!["RENAME_BOTH"]);
  key_match!(rename_t, "RenameT.tsx".into(), vec!["RENAME_T"]);
  key_match!(
    rename_use_translation,
    "RenameUseTranslation.tsx".into(),
    vec!["RENAME_USE_TRANSLATION"]
  );

  key_match!(
    warp_use_translation_ns,
    "WrapUseTranslationNs/Component.tsx".into(),
    "namespace_3".into(),
    vec!["WRAPPED_USE_TRANSLATION_NS"]
  );

  key_match!(member_t, "memberT.ts".into(), vec!["MEMBER_T"]);

  key_match!(
    member_call_t,
    "MemberCallT.tsx".into(),
    vec!["MEMBER_CALL_T"]
  );

  key_match!(
    i18n_code_from_string_literal,
    "I18nCodeFromStringLiteral.tsx".into(),
    vec!["I18N_CODE_FROM_STRING_LITERAL"]
  );

  key_match!(
    namespace_from_variable,
    "NamespaceFromVar.tsx".into(),
    "namespace_3".into(),
    vec!["NAMESPACE_FROM_VAR"]
  );

  key_match!(
    i18n_code_from_template_literal,
    "I18nCodeFromTemplateLiteral.tsx".into(),
    vec!["I18N_CODE_FROM_TEMPLATE_LITERAL"]
  );

  key_match!(
    i18n_code_cross_file,
    "I18nCodeCrossFile/Component.tsx".into(),
    vec!["I18N_CODE_CROSS_FILE"]
  );

  key_match!(
    namespace_import,
    "NamespaceImport.tsx".into(),
    vec!["NAMESPACE_IMPORT"]
  );

  key_match!(
    wrap_use_translation,
    "WrapUseTranslation/Component.tsx".into(),
    vec!["WRAPPED_USE_TRANSLATION"]
  );

  key_match!(hoc_component, "HocComp.tsx".into(), vec!["HOC_COMPONENT"]);

  key_match!(
    trans_component,
    "TransComp.tsx".into(),
    vec!["TRANS_COMPONENT"]
  );

  key_match!(
    translation_component,
    "TranslationComp.tsx".into(),
    vec!["TRANSLATION_COMPONENT"]
  );

  key_match!(
    i18n_code_dynamic,
    "I18nCodeDynamic.tsx".into(),
    vec!["I18N_CODE_DYNAMIC_hello", "I18N_CODE_DYNAMIC_world"]
  );

  key_match!(
    i18n_hook_inline,
    "CustomHookInline.tsx".into(),
    "namespace_4".into(),
    vec![I18nPackage {
      package_path: "@custom/i18n".into(),
      members: vec![Member {
        ns: None,
        name: "useTranslation".into(),
        r#type: I18nType::Hook
      }]
    }],
    vec!["CUSTOM_HOOK_INLINE"]
  );

  #[test]
  fn collect_custom_i18n_package() {
    let extend = make_custom_i18n_package();
    let (_, collector) = collect("custom-i18n/index.tsx".into(), Some(extend));

    assert_eq!(collector.i18n_namespaces.len(), 4);
    assert_eq!(collector.get_keys("default").len(), 16);
    assert_eq!(collector.get_keys("namespace_1").len(), 2);
    assert_eq!(collector.get_keys("namespace_2").len(), 1);
    assert_eq!(collector.get_keys("namespace_3").len(), 2);
  }

  #[test]
  fn collect_custom_i18n_with_externals() {
    let extend = make_custom_i18n_package();
    let (_, collector) = collect_with_options(
      "custom-i18n/index.tsx".into(),
      Some(extend),
      vec![
        "@custom/i18n".into(),
        "i18next".into(),
        "react-i18next".into(),
      ],
    );

    assert_eq!(collector.get_keys("default").len(), 16);
    assert_eq!(collector.get_keys("namespace_3").len(), 2);
  }
}
