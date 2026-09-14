use super::i18n_types::{I18nMember, I18nType};
use super::node::{ExportBinding, Node, NodePath};
use super::node_store::NodeStore;
use std::collections::HashSet;

enum Resolution {
  Missing,
  Binding(NodePath, String, Option<I18nMember>),
  Ambiguous,
}

impl Node {
  pub fn get_export_origin_name(&self, name: &str) -> Option<String> {
    match self.resolve_export(name, &mut HashSet::new()) {
      Resolution::Binding(_, name, _) => Some(name),
      _ => None,
    }
  }

  fn exported_names(&self, visited: &mut HashSet<NodePath>) -> HashSet<String> {
    if !visited.insert(self.file_path.clone()) {
      return HashSet::new();
    }
    let mut names: HashSet<_> = self.export_bindings.borrow().keys().cloned().collect();
    for source in self.reexport_all.borrow().iter() {
      if let Some(node) = self.get_importing_node(source) {
        names.extend(
          node
            .exported_names(visited)
            .into_iter()
            .filter(|name| name != "default"),
        );
      }
    }
    names
  }

  fn resolve_binding(
    &self,
    name: &str,
    binding: &ExportBinding,
    visiting: &mut HashSet<(NodePath, String)>,
  ) -> Resolution {
    match binding {
      ExportBinding::Local(member) => {
        Resolution::Binding(self.file_path.clone(), name.to_string(), member.clone())
      }
      ExportBinding::LocalAlias { name, binding } => self.resolve_binding(name, binding, visiting),
      ExportBinding::Imported { source, name } => self
        .get_importing_node(source)
        .map_or(Resolution::Missing, |node| {
          node.resolve_export(name, visiting)
        }),
      ExportBinding::Wrapper { callee, ns } => {
        let member = match self.resolve_binding(name, callee, visiting) {
          Resolution::Binding(_, _, Some(mut member)) => {
            // A hook's explicit namespace overrides the provider namespace. Translation
            // calls take a key as their first argument, not a namespace.
            if matches!(member.r#type, I18nType::Hook) {
              member.ns = ns.clone().or(member.ns);
            }
            Some(member)
          }
          _ => None,
        };
        Resolution::Binding(self.file_path.clone(), name.to_string(), member)
      }
    }
  }

  fn resolve_export(&self, name: &str, visiting: &mut HashSet<(NodePath, String)>) -> Resolution {
    let key = (self.file_path.clone(), name.to_string());
    if !visiting.insert(key.clone()) {
      return Resolution::Missing;
    }
    // Explicit exports shadow stars, including explicitly non-i18n bindings.
    let result = if let Some(binding) = self.export_bindings.borrow().get(name) {
      self.resolve_binding(name, binding, visiting)
    } else if name == "default" {
      Resolution::Missing
    } else {
      let mut result = Resolution::Missing;
      for source in self.reexport_all.borrow().iter() {
        let Some(node) = self.get_importing_node(source) else {
          continue;
        };
        let candidate = node.resolve_export(name, visiting);
        match (&result, &candidate) {
          (_, Resolution::Missing) => {}
          (Resolution::Missing, _) => result = candidate,
          (
            Resolution::Binding(path, export, _),
            Resolution::Binding(other_path, other_export, _),
          ) if path == other_path && export == other_export => {}
          _ => {
            result = Resolution::Ambiguous;
            break;
          }
        }
      }
      result
    };
    visiting.remove(&key);
    result
  }
}

impl NodeStore {
  pub fn resolve_exports(&self) {
    let nodes = self.get_all_nodes();
    for node in nodes.values() {
      let members = node
        .exported_names(&mut HashSet::new())
        .into_iter()
        .map(|name| {
          // Resolve each binding against the complete graph. Do not cache a miss from
          // a partially visited cycle: another path can still reach its provider.
          let member = match node.resolve_export(&name, &mut HashSet::new()) {
            Resolution::Binding(_, _, member) => member,
            _ => None,
          };
          (name, member)
        })
        .collect();
      node.set_resolved_exports(members);
    }
    // Both new and previously visited dependencies use the same member-level rule.
    for node in nodes.values() {
      node.resolve_i18n_imports();
    }
  }
}
