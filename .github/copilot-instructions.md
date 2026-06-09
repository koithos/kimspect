# Copilot instructions for `kimspect`

## Build, test, and lint

- Use the pinned Rust toolchain from `rust-toolchain.toml` (`1.90.0` with `rustfmt` and `clippy`).
- Build locally with `cargo build` or `cargo build --release`.
- CI lint commands are:
  - `cargo clippy -- -D warnings`
  - `cargo fmt --all -- --check --color always`
- The pre-commit config also runs `cargo check` and `cargo clippy --fix --no-deps --allow-dirty --allow-staged`.
- Run the full test suite with `cargo test`.
- Run a single test by name with `cargo test test_process_pod`.
- Run a single integration-test target and test with `cargo test --test cli_tests test_cli_parse_get_images_default`.
- `tests/integration_tests.rs` uses a real Kubernetes client. CI starts a Kind cluster before `cargo test`, so cluster-dependent tests expect working kube access rather than mocks.

## High-level architecture

- `src/main.rs` is the binary entrypoint. It parses Clap args, initializes tracing, creates `K8sClient`, and dispatches the selected subcommand.
- `src/lib.rs` re-exports the main CLI types, Kubernetes logic, and display helpers so the binary and integration tests use the same public API.
- CLI definitions are split across `src/cli/args.rs`, `src/cli/commands.rs`, and `src/cli/formats.rs`:
  - `Args` owns global flags like verbosity, log format, and request timeout.
  - `Commands` / `GetImages` define the `get images` and `get registries` subcommands and their conflict rules.
  - `OutputFormat` controls whether rendering stays in `normal` mode or expands to `wide`.
- `src/k8s/mod.rs` is the core domain layer:
  - `K8sClient` creates the `kube` client, checks cluster accessibility, validates namespaces, and fetches Kubernetes resources.
  - `get_pod_images` lists pods, applies node/pod/registry filters, converts pods into `PodImage` records, then does a second pass over Node status to enrich image sizes from digests.
  - `get_unique_registries` does **not** scan pods; it scans Deployment container specs and returns sorted unique registries.
  - Image parsing lives in `extract_registry`, `split_image`, and `process_pod`.
- `src/utils/mod.rs` is the presentation layer. It renders `PodImage` rows with `prettytable`; `wide` output is where registry, size, digest, and node columns are added.
- `src/utils/logging.rs` owns tracing setup and maps `-v` counts to log levels.

## Key conventions

- Reuse the existing image-parsing helpers in `src/k8s/mod.rs` instead of reimplementing registry/tag/digest parsing. `tests/k8s_tests.rs` covers many edge cases, including registries with ports and digest-only image refs.
- `PodImage` is the shared shape between Kubernetes collection and CLI rendering. If you add or rename image metadata, update both `process_pod` / `get_pod_images` and the table rendering in `src/utils/mod.rs`.
- Preserve the current command/data-source split:
  - `get images` is pod-based.
  - `get registries` is deployment-based.
  Changes that assume both commands read the same Kubernetes resource will be wrong.
- Namespace handling is explicit. `K8sClient` checks namespace existence unless `--all-namespaces` is set, and empty results are often turned into `K8sError::ResourceNotFound` rather than silently returning an empty success.
- Image size enrichment is best-effort. If listing Nodes fails, `get_pod_images` still returns image rows, just without size data.
- CLI argument behavior is enforced in tests. When changing flags, keep the conflict/compatibility rules aligned with `tests/cli_tests.rs`:
  - `--namespace` conflicts with `--all-namespaces`
  - `--registry` conflicts with `--exclude-registry`
  - `-o wide` enables the extra table columns
- Logging follows the existing `tracing` + `anyhow::Context` + `#[instrument]` pattern. Match that style when adding new async Kubernetes operations.
- The CLI currently parses `--kubeconfig`, but `K8sClient::new` still resolves kubeconfig from `KUBECONFIG` or `~/.kube/config`. If you work on kubeconfig handling, update both the CLI surface and the client wiring.
- Formatting is intentionally opinionated: `rustfmt.toml` sets `max_width = 100` and related layout rules, and CI treats formatting drift as a failure.
