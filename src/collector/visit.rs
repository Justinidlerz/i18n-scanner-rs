use super::walker::Walker;
use crate::node::i18n_types::I18nType;
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};
use oxc_ast_visit::Visit;

impl<'a> Visit<'a> for Walker<'a> {
  fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
    if it.import_kind.is_type() {
      return;
    }
    let Some(specifiers) = &it.specifiers else {
      return;
    };
    let Some(source) = self.node.get_importing_node(&it.source.value) else {
      return;
    };
    let members = source.get_exporting_members();
    for specifier in specifiers {
      let (name, local) = match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(s) if !s.import_kind.is_type() => {
          (s.imported.name().to_string(), &s.local)
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => ("default".into(), &s.local),
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
          self.register_translation_names(Walker::collect_t_member_names(&members));
          self.read_namespace_import(s.local.symbol_id(), &members);
          continue;
        }
        _ => continue,
      };
      // The analyzer resolved the binding, including reexports. A familiar local
      // name alone must not turn an unrelated import into an i18n provider.
      let Some(Some(member)) = members.get(&name) else {
        continue;
      };
      match member.r#type {
        I18nType::Hook => {
          let origin_name = source.get_export_origin_name(&name).unwrap_or(name);
          self.read_hook(local.symbol_id(), &origin_name, member.ns.clone(), &members);
        }
        I18nType::TMethod => {
          self.register_t_symbol(local.symbol_id(), local.name.as_str());
          self.read_t(local.symbol_id(), member.ns.clone());
        }
        I18nType::ObjectMemberT => {
          self.register_translation_names(Walker::collect_t_member_names(&members));
          self.read_object_member_t(local.symbol_id(), member.ns.clone());
        }
        I18nType::TransComp => self.read_trans_component(
          local.symbol_id(),
          member.ns.clone(),
          member.key_prop.clone(),
        ),
        I18nType::TranslationComp => {
          self.read_translation_component(local.symbol_id(), member.ns.clone())
        }
        I18nType::HocWrapper => self.read_hoc_wrapper(local.symbol_id(), member.ns.clone()),
      }
    }
  }
}
