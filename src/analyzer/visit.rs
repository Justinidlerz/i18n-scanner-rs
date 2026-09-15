use super::walker::Walker;
use crate::analyzer::i18n_packages::preset_member_type;
use crate::node::i18n_types::I18nMember;
use crate::node::node::ExportBinding;
use oxc_ast::ast::{
  ArrayPattern, BindingPattern, Declaration, ExportAllDeclaration, ExportDefaultDeclaration,
  ExportDefaultDeclarationKind, ExportNamedDeclaration, Expression, ImportDeclaration,
  ImportDeclarationSpecifier, ImportExpression, ModuleExportName, ObjectPattern,
};
use oxc_ast_visit::Visit;
use std::collections::HashSet;

impl<'a> Visit<'a> for Walker<'a> {
  // import('xyz')
  fn visit_import_expression(&mut self, it: &ImportExpression<'a>) {
    match &it.source {
      Expression::StringLiteral(source) => {
        // we assume doesn't import 'i18next' from other packages
        // doesn't handle dynamic import specifiers
        self.resolve_import(source, vec![]);
      }
      _ => {}
    }
  }

  // import './xyz'
  // import xyz from './xyz'
  // import { x, y, z } from './xyz'
  fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
    if it.import_kind.is_type() {
      return;
    }
    let specifiers = match &it.specifiers {
      Some(specifiers) => Some(
        specifiers
          .iter()
          .filter_map(|specifier| match specifier {
            // import { foo } from 'xyz'
            ImportDeclarationSpecifier::ImportSpecifier(s) => {
              (!s.import_kind.is_type()).then(|| s.imported.name().to_string())
            }
            // import * as xyz from 'xyz'
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => Some("*".into()),
            // import xyz from 'xyz'
            ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => Some("default".into()),
          })
          .collect(),
      ),
      None => None,
    };

    self.resolve_import(&it.source, specifiers.unwrap_or_default())
  }

  // export { foo } from './xyz';
  // export const a = "xyz";
  fn visit_export_named_declaration(&mut self, it: &ExportNamedDeclaration<'a>) {
    if it.export_kind.is_type() {
      // Type-only exports never create runtime bindings, so there is nothing to resolve for i18n.
      return;
    }

    self.has_exports = true;
    if let Some(source) = &it.source {
      let specs = it
        .specifiers
        .iter()
        .filter(|s| !s.export_kind.is_type())
        .map(|s| s.local.name().to_string())
        .collect();
      self.resolve_import(source, specs);
      if self.collect_exports {
        let exports = it
          .specifiers
          .iter()
          .filter(|s| !s.export_kind.is_type())
          .map(|s| {
            (
              s.exported.name().to_string(),
              ExportBinding::Imported {
                source: source.value.to_string(),
                name: s.local.name().to_string(),
              },
            )
          })
          .collect();
        self.append_exports(exports);
      }
    } else if self.collect_exports {
      let exports = it
        .specifiers
        .iter()
        .filter(|s| !s.export_kind.is_type())
        .map(|s| {
          let binding = match &s.local {
            ModuleExportName::IdentifierReference(ident) => self.resolve_export_identifier(ident),
            _ => ExportBinding::Local(None),
          };
          (s.exported.name().to_string(), binding)
        })
        .collect();
      self.append_exports(exports);
    }
    if !self.collect_exports {
      return;
    }

    // export const a = "xyz";
    if let Some(declaration) = &it.declaration {
      let specs = match &declaration {
        // export function a() {}
        Declaration::FunctionDeclaration(decl) => {
          if let Some(ident) = &decl.id {
            let name = ident.name.to_string();
            // Recognize well-known i18n exports even when they are implemented directly.
            let member = preset_member_type(name.as_str()).map(|member_type| I18nMember {
              r#type: member_type,
              ns: None,
              key_prop: None,
            });

            vec![(name, ExportBinding::Local(member))]
          } else {
            vec![]
          }
        }
        Declaration::VariableDeclaration(decl) => decl
          .declarations
          .iter()
          .flat_map(|de| {
            match &de.id {
              // export const a = "xyz";
              BindingPattern::BindingIdentifier(ident) => {
                vec![(ident.name.to_string(), self.resolve_i18n_export(de))]
              }
              // export const { a, b } = xyz;
              BindingPattern::ObjectPattern(obj) => collect_deconstructed_object_export(&obj),
              // export const [a, b] = xyz;
              BindingPattern::ArrayPattern(arr) => collect_deconstructed_array_export(&arr),
              _ => vec![],
            }
          })
          .collect(),
        _ => {
          vec![]
        }
      };
      self.append_exports(specs);
    }
  }

  fn visit_export_default_declaration(&mut self, it: &ExportDefaultDeclaration<'a>) {
    self.has_exports = true;
    if !self.collect_exports {
      return;
    }
    let binding = match &it.declaration {
      ExportDefaultDeclarationKind::Identifier(ident) => self.resolve_export_identifier(ident),
      _ => it
        .declaration
        .as_expression()
        .and_then(|expr| self.resolve_export_expression(expr, &mut HashSet::new()))
        .unwrap_or(ExportBinding::Local(None)),
    };
    self.append_exports(vec![("default".into(), binding)]);
  }
  // export * from './xyz';
  fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
    if it.export_kind.is_type() {
      return;
    }
    self.has_exports = true;
    self.resolve_import(&it.source, vec!["*".into()]);
    if self.collect_exports {
      if let Some(exported) = &it.exported {
        // A namespace export does not flatten the source module's members.
        self.append_exports(vec![(
          exported.name().to_string(),
          ExportBinding::Local(None),
        )]);
      } else {
        self.append_reexport(&it.source);
      }
    }
  }
}

fn collect_deconstructed_array_export(arr: &ArrayPattern) -> Vec<(String, ExportBinding)> {
  arr
    .elements
    .iter()
    .filter_map(|element| {
      // Skip empty slots in the array pattern (e.g. [, value])
      let Some(pattern) = element.as_ref() else {
        return None;
      };

      match pattern {
        BindingPattern::BindingIdentifier(ident) => {
          Some((ident.name.to_string(), ExportBinding::Local(None)))
        }
        _ => None,
      }
    })
    .collect()
}

fn collect_deconstructed_object_export(obj: &ObjectPattern) -> Vec<(String, ExportBinding)> {
  if obj.properties.is_empty() {
    return vec![];
  }

  obj
    .properties
    .iter()
    .fold(vec![], |acc, prop| match &prop.value {
      BindingPattern::ObjectPattern(obj) => {
        [collect_deconstructed_object_export(&obj), acc].concat()
      }
      BindingPattern::BindingIdentifier(ident) => [
        vec![(ident.name.to_string(), ExportBinding::Local(None))],
        acc,
      ]
      .concat(),
      _ => acc,
    })
}
