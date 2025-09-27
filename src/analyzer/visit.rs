use super::walker::Walker;
use crate::node::i18n_types::I18nMember;
use oxc_ast::ast::{
  ArrayPattern, BindingPatternKind, Declaration, ExportAllDeclaration, ExportDefaultDeclaration,
  ExportNamedDeclaration, Expression, ImportDeclaration, ImportDeclarationSpecifier,
  ImportExpression, ModuleExportName, ObjectPattern,
};
use oxc_ast::AstKind;
use oxc_ast_visit::Visit;

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
    let specifiers = match &it.specifiers {
      Some(specifiers) => Some(
        specifiers
          .iter()
          .filter_map(|specifier| match specifier {
            // import { foo } from 'xyz'
            ImportDeclarationSpecifier::ImportSpecifier(s) => Some(s.imported.name().to_string()),
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
    if it.specifiers.len() > 0 {
      // export { foo } from './xyz';
      if let Some(source) = &it.source {
        let specs = it
          .specifiers
          .iter()
          .map(|specifier| specifier.exported.name().to_string())
          .collect::<Vec<String>>();
        self.resolve_import(source, specs.clone());
      } else {
        // export { abc };
        let exports = it
          .specifiers
          .iter()
          .map(|specifier| match &specifier.local {
            ModuleExportName::IdentifierReference(ident) => {
              let Some(node) = self
                .walk_utils
                .get_var_defined_node(ident.reference_id.get().unwrap())
              else {
                return (specifier.exported.name().to_string(), None);
              };

              match node.kind() {
                AstKind::VariableDeclarator(decl) => {
                  return (
                    specifier.exported.name().to_string(),
                    self.resolve_i18n_export(&decl),
                  )
                }
                _ => (specifier.exported.name().to_string(), None),
              }
            }
            _ => (specifier.exported.name().to_string(), None),
          })
          .collect::<Vec<(String, Option<I18nMember>)>>();

        self.append_exports(exports);
      }
    }

    // export const a = "xyz";
    if let Some(declaration) = &it.declaration {
      let specs = match &declaration {
        // export function a() {}
        Declaration::FunctionDeclaration(_decl) => {
          // vec![(decl.id.name.to_string(), self.resolve_i18n_export(decl))]
          // TODO: handle function declaration
          vec![]
        }
        Declaration::VariableDeclaration(decl) => decl
          .declarations
          .iter()
          .flat_map(|de| {
            match &de.id.kind {
              // export const a = "xyz";
              BindingPatternKind::BindingIdentifier(ident) => {
                vec![(ident.name.to_string(), self.resolve_i18n_export(de))]
              }
              // export const { a, b } = xyz;
              BindingPatternKind::ObjectPattern(obj) => collect_deconstructed_object_export(&obj),
              // export const [a, b] = xyz;
              BindingPatternKind::ArrayPattern(arr) => collect_deconstructed_array_export(&arr),
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

  fn visit_export_default_declaration(&mut self, _: &ExportDefaultDeclaration<'a>) {
    // TODO collect i18n_member the default export when it's a function
    self.append_exports(vec![("default".into(), None)])
  }
  // export * from './xyz';
  fn visit_export_all_declaration(&mut self, it: &ExportAllDeclaration<'a>) {
    self.resolve_import(&it.source, vec!["*".into()]);
    self.append_reexport(&it.source);
  }
}

fn collect_deconstructed_array_export(arr: &ArrayPattern) -> Vec<(String, Option<I18nMember>)> {
  arr
    .elements
    .iter()
    .filter_map(|ele| {
      let ele_ref = ele.as_ref();

      match &ele_ref.unwrap().kind {
        BindingPatternKind::BindingIdentifier(ident) => Some((ident.name.to_string(), None)),
        _ => None,
      }
    })
    .collect()
}

fn collect_deconstructed_object_export(obj: &ObjectPattern) -> Vec<(String, Option<I18nMember>)> {
  if obj.properties.is_empty() {
    return vec![];
  }

  obj
    .properties
    .iter()
    .fold(vec![], |acc, prop| match &prop.value.kind {
      BindingPatternKind::ObjectPattern(obj) => {
        [collect_deconstructed_object_export(&obj), acc].concat()
      }
      BindingPatternKind::BindingIdentifier(ident) => {
        [vec![(ident.name.to_string(), None)], acc].concat()
      }
      _ => acc,
    })
}
