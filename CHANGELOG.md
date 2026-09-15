# @i18n-scanner-rs/main

## 0.3.1

### Patch Changes

- 3c2de48: Make scanned key sets independent of dependency traversal order by resolving imports, aliases, and reexports against the complete module graph, including cycles, while preserving namespace and keyProp metadata.

## 0.3.0

### Minor Changes

- d3cb81d: Support absolute extended i18n package paths, custom `TransComp` key props, and translation calls in emitted ESM and CommonJS modules.

### Patch Changes

- 92be256: Add Changesets-based beta and stable package publishing workflows.
