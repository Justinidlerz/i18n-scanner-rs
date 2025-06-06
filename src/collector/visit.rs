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
                    self.read_hook(s, member.ns.clone());
                  }
                  I18nType::TMethod => {
                    self.read_t(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::ObjectMemberT => {
                    self.read_object_member_t(s.local.symbol_id(), member.ns.clone());
                  }
                  I18nType::TransComp => {}
                  I18nType::TranslationComp => {}
                  I18nType::HocWrapper => {}
                }
              }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {
              // TODO: Handle namespace import,
              //       find out the namespace members and handle it like import specifier
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
