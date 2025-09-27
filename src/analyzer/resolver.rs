use log::debug;
use oxc_resolver::TsconfigReferences::Auto;
use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions};
use std::path::PathBuf;

pub fn create_resolver(tsconfig_path: String) -> Resolver {
  debug!("tsconfig_path: {}", tsconfig_path);

  Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), ".tsx".into(), ".js".into(), ".jsx".into()],
    // ESM
    condition_names: vec!["import".into(), "default".into(), "module".into()],
    tsconfig: Option::from(TsconfigOptions {
      config_file: PathBuf::from(tsconfig_path),
      references: Auto,
    }),
    ..ResolveOptions::default()
  })
}
