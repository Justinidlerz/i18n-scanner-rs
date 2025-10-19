use super::walker::Walker;
use crate::node::i18n_types::{I18nMember, I18nType};
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};
use oxc_ast_visit::Visit;
use std::collections::HashMap;

impl<'a> Visit<'a> for Walker<'a> {
  fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
    if let Some(specifiers) = &it.specifiers {
      if let Some(i18n_source) = self.node.get_importing_node(&it.source.value) {
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
              } else if self.is_standard_hook_export(s.imported.name().as_str()) {
                let empty_members: HashMap<String, Option<I18nMember>> = HashMap::new();
                let translation_names = Walker::collect_t_member_names(&empty_members);
                self.register_translation_names(translation_names.clone());
                self.read_hook(s, None, &empty_members);
              } else if s.imported.name().as_str() == "t" {
                self.register_t_symbol(s.local.symbol_id(), s.local.name.as_str());
                self.read_t(s.local.symbol_id(), None);
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
      } else {
        for specifier in specifiers {
          if let ImportDeclarationSpecifier::ImportSpecifier(s) = specifier {
            let imported_name = s.imported.name().as_str();
            if self.is_standard_hook_export(imported_name) {
              let empty_members: HashMap<String, Option<_>> = HashMap::new();
              let translation_names = Walker::collect_t_member_names(&empty_members);
              self.register_translation_names(translation_names.clone());
              self.read_hook(s, None, &empty_members);
            } else if imported_name == "t" {
              self.register_t_symbol(s.local.symbol_id(), s.local.name.as_str());
              self.read_t(s.local.symbol_id(), None);
            }
          }
        }
      }
    }
  }

  // fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
  //   todo!()
  // }
}
