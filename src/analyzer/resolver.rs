use log::debug;
use oxc_resolver::{
  ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
};
use std::path::PathBuf;

pub fn create_resolver(tsconfig_path: String) -> Resolver {
  debug!("tsconfig_path: {}", tsconfig_path);

  let tsconfig = if tsconfig_path.trim().is_empty() {
    // Without a provided tsconfig we fall back to the default node resolution behavior.
    None
  } else {
    Some(TsconfigDiscovery::Manual(TsconfigOptions {
      // When a tsconfig path is supplied we resolve exactly that configuration file.
      config_file: PathBuf::from(tsconfig_path),
      references: TsconfigReferences::Auto,
    }))
  };

  Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), ".tsx".into(), ".js".into(), ".jsx".into()],
    // ESM
    condition_names: vec!["import".into(), "default".into(), "module".into()],
    tsconfig,
    ..ResolveOptions::default()
  })
}
