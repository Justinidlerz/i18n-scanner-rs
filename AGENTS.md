## Project map

- **Entry point** – `src/lib.rs` exposes the `scan` N-API function, seeds i18n packages, runs the analyzer across every entry path, and hands the shared `NodeStore` to the collector.

- **Static analysis** – `src/analyzer/` owns module resolution and graph construction. `Analyzer` drives parsing and recursion, `walker.rs` inspects AST nodes, and `i18n_packages.rs` resolves preset/custom packages before analysis.

- **Key collection** – `src/collector/` traverses analyzed modules, uses `WalkerUtils` for string resolution, and aggregates keys by namespace via `collector::Walker`. Extend walkers here when adding new extraction heuristics.

- **Graph storage** – `src/node/` defines `Node` and `NodeStore`, tracking import/export metadata and whether a module exposes or consumes i18n helpers.

- **Fixture project** – Add real-world TypeScript cases under `tests/fake-project/src` so both Rust unit tests and Vitest snapshots cover new behavior.

## Workflow expectations

1. When changing detection logic, update the relevant analyzer or collector walker and ensure `WalkerUtils` can resolve any new string patterns before relying on ad-hoc parsing.

2. For new i18n providers, register them through `Analyzer::extend_i18n_packages` so they resolve alongside presets instead of hard-coding paths.

3. Mirror every feature addition in the Vitest snapshot or dedicated Rust tests to keep the JS binding and Rust core in sync; note how snapshots sort keys before comparison for determinism.

## Testing

- Run `cargo test` for the Rust crate before committing.

- Run `pnpm test` (Vitest) to verify the JavaScript binding behavior; this exercises the published API against the fixture project.

- Build artifacts with `pnpm build` when you need to validate the N-API bundle locally.

## Formatting & linting

- Always format Rust sources with `cargo fmt` (also available via `pnpm format:rs`) and TypeScript/JavaScript with `pnpm format:prettier` (or `pnpm format`) before committing.

- Keep comments in English on critical control-flow branches so reviewers can follow new heuristics; align with the existing inline guidance in walkers.

- Maintain sorted key output in tests (see `tests.spec.ts`) to avoid snapshot churn.

## Additional notes

- Prefer extending existing helpers (e.g., `WalkerUtils::read_str_expression`) over introducing parallel logic to keep expression handling centralized.

- When linking modules manually, use the `NodeStore` APIs (`try_insert_importing`, `insert_exporting`) so downstream collectors see consistent metadata.
