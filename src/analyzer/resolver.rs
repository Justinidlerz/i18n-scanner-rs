use log::debug;
use oxc_resolver::{
  ResolveOptions, Resolver, TsconfigDiscovery, TsconfigOptions, TsconfigReferences,
};
use std::path::PathBuf;

pub fn create_resolver(tsconfig_path: String) -> Resolver {
  debug!("tsconfig_path: {}", tsconfig_path);

  let tsconfig_file = (!tsconfig_path.trim().is_empty()).then(|| {
    let raw = PathBuf::from(tsconfig_path);
    if raw.is_absolute() {
      raw
    } else {
      std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(raw)
    }
  });
  let cwd = tsconfig_file
    .as_ref()
    .and_then(|config_file| config_file.parent().map(|parent| parent.to_path_buf()));
  let tsconfig = tsconfig_file.map(|config_file| {
    TsconfigDiscovery::Manual(TsconfigOptions {
      // When a tsconfig path is supplied we resolve exactly that configuration file.
      config_file,
      references: TsconfigReferences::Auto,
    })
  });

  Resolver::new(ResolveOptions {
    extensions: vec![".ts".into(), ".tsx".into(), ".js".into(), ".jsx".into()],
    // ESM
    condition_names: vec!["import".into(), "default".into(), "module".into()],
    cwd,
    tsconfig,
    ..ResolveOptions::default()
  })
}

#[cfg(test)]
mod tests {
  use super::create_resolver;
  use std::fs;
  use std::path::PathBuf;
  use std::time::{SystemTime, UNIX_EPOCH};

  fn make_temp_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock drift")
      .as_nanos();
    std::env::temp_dir().join(format!("i18n-scanner-rs-{name}-{nanos}"))
  }

  #[test]
  fn resolves_alias_from_referenced_tsconfig() {
    let root = make_temp_root("resolver");
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src directory");

    fs::write(root.join("tsconfig.base.json"), r#"{"compilerOptions":{}}"#)
      .expect("write tsconfig.base.json");
    fs::write(
      root.join("tsconfig.app.json"),
      r#"{
  "extends": "./tsconfig.base.json",
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}"#,
    )
    .expect("write tsconfig.app.json");
    fs::write(
      root.join("tsconfig.json"),
      r#"{
  "extends": "./tsconfig.base.json",
  "references": [{ "path": "./tsconfig.app.json" }]
}"#,
    )
    .expect("write tsconfig.json");
    fs::write(src_dir.join("foo.ts"), "export const foo = 1;\n").expect("write foo.ts");

    let resolver = create_resolver(root.join("tsconfig.json").to_string_lossy().to_string());
    let resolution = resolver
      .resolve(src_dir.as_path(), "@/foo")
      .expect("resolve @/foo from src");
    let resolved = resolution.path();

    assert!(
      resolved.ends_with("src/foo.ts"),
      "expected alias to resolve to src/foo.ts, got {}",
      resolved.display()
    );

    fs::remove_dir_all(&root).expect("cleanup temp test directory");
  }
}
