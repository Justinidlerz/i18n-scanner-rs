use crate::analyzer::analyzer::Analyzer;
use crate::node::i18n_types::{I18nMember, I18nType};
use crate::node::node::Node;
use std::path::Path;
use std::rc::Rc;

///
/// We assume those packages includes the below methods exposed
/// - t: global t method
/// - useTranslation: the t method warped as a React hook
/// - Trans: the i18n component
/// - Translation: the i18n component
/// - withTranslation: the i18n HOC function
///
/// and we don't handle other exposed methods from this package
pub static PRESET_I18N_PACKAGES: &[&str] = &[
  "i18next",
  "react-i18next",
];

pub static PRESET_I18N_MEMBERS: &[(&str, I18nType)] = &[
  // Special symbol import as namespace
  ("t", I18nType::TMethod),
  ("useTranslation", I18nType::Hook),
  ("Trans", I18nType::TransComp),
  ("Translation", I18nType::TranslationComp),
  ("withTranslation", I18nType::HocWrapper),
  ("i18n", I18nType::ObjectMemberT),
];

#[derive(Clone)]
#[napi(object)]
pub struct Member {
  pub name: String,
  pub r#type: I18nType,
  pub ns: Option<String>,
}
#[derive(Clone)]
#[napi(object)]
pub struct I18nPackage {
  pub package_path: String,
  pub members: Vec<Member>,
}

impl Analyzer {
  pub fn seed(
    &mut self,
    entry_path: &str,
    extend_i18n_packages: Option<Vec<I18nPackage>>,
  ) -> &mut Self {
    let i18n_packages = self.extend_i18n_packages(entry_path, extend_i18n_packages);
    for package in i18n_packages {
      let file_path_ref = Rc::new(package.package_path);
      let node = Rc::new(Node::new(file_path_ref.clone(), self.node_store.clone()));

      for member in package.members {
        node.insert_exporting(
          member.name,
          Some(I18nMember {
            r#type: member.r#type,
            ns: member.ns,
          }),
        );
      }

      self.node_store.insert_node(file_path_ref, node);
    }

    self
  }

  pub fn make_preset_i18n_packages(&self, basename: &Path) -> Vec<I18nPackage> {
    let mut methods: Vec<I18nPackage> = Vec::new();

    for pkg_name in PRESET_I18N_PACKAGES {
      // resolve packages and confirm it's installed
      if let Ok(res) = self.resolver.resolve(basename, pkg_name) {
        if let Some(path_str) = res.path().to_str() {
          let mut members = Vec::new();
          for (name, r#type) in PRESET_I18N_MEMBERS {
            members.push(Member {
              name: name.to_string(),
              r#type: r#type.clone(),
              ns: None,
            })
          }

          methods.push(I18nPackage {
            package_path: path_str.to_string(),
            members,
          })
        }
      }
    }
    methods
  }

  pub fn extend_i18n_packages(
    &self,
    entry_path: &str,
    extend_packages: Option<Vec<I18nPackage>>,
  ) -> Vec<I18nPackage> {
    let basename = Path::new(entry_path)
        .parent()
        .unwrap_or_else(|| Path::new("."));
    
    let mut packages = extend_packages.and_then(|packages| {
      let mut pkgs: Vec<I18nPackage> = vec![];
      for pkg in &packages {
        if let Ok(res) = self.resolver.resolve(basename, &pkg.package_path) {
          if let Some(path_str) = res.path().to_str() {
            pkgs.push(I18nPackage {
              package_path: path_str.to_string(),
              members: pkg.members.clone(),
            });
          }
        }
      }
      Some(pkgs)
    }).unwrap_or_else(|| Vec::new());

    packages.extend(self.make_preset_i18n_packages(basename));

    packages
  }
}
