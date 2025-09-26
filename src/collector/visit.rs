use super::walker::Walker;
use crate::node::i18n_types::I18nType;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};
use oxc_ast_visit::Visit;

impl<'a> Visit<'a> for Walker<'a> {
  fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
    if let Some(i18n_source) = self.node.get_importing_node(&it.source.value) {
      if let Some(specifiers) = &it.specifiers {
        let members = i18n_source.get_exporting_members();
        for specifier in specifiers {
          match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
              if let Some(Some(member)) = members.get(s.imported.name().as_str()) {
                match member.r#type {
                  I18nType::Hook => {
                    self.read_hook(s, member.ns.clone(), &members);
                  }
                  I18nType::TMethod => {
                    self.register_t_symbol(s.local.symbol_id(), s.local.name.as_str());
                    self.read_t(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::ObjectMemberT => {
                    let translation_names = Walker::collect_t_member_names(&members);
                    self.register_translation_names(translation_names);
                    self.read_object_member_t(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::TransComp => {
                    self.read_trans_component(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::TranslationComp => {
                    self.read_translation_component(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::HocWrapper => {
                    self.read_hoc_wrapper(s.local.symbol_id(), member.ns.clone());
                  }
                }
              }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns_spec) => {
              // Handle namespace import
              // For namespace imports like `import * as i18n from 'react-i18next'`
              // We need to handle calls like `i18n.useTranslation()` and `i18n.t()`
              if let Some(i18n_source) = self.node.get_importing_node(&it.source.value) {
                let members = i18n_source.get_exporting_members();
                let translation_names = Walker::collect_t_member_names(&members);
                self.register_translation_names(translation_names);
                // Handle the namespace symbol
                self.read_namespace_import(ns_spec.local.symbol_id(), &members);
              }
            }
            _ => {}
          }
        }
      }
    }
  }

  // fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
  //   todo!()
  // }
}
