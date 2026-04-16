# synthesizer-core

Shared typed-AST primitives **and the provable artifact hierarchy** for the
pleme-io synthesizer family. Wave 1 of the "compound knowledge" refactor.
Purely additive — no breaking changes to any existing synthesizer.

## The claim

> The big deal here is we render artifacts provably and reliably through
> typescape and AST.

```text
Rust types → typed AST (SynthesizerNode) → bytes → Artifact → disk / registry / cluster
```

Every link is deterministic, every link is under tests, every artifact is
content-addressable by construction.

## Tests

| Module | Purpose | Tests |
|--------|---------|-------|
| `node` | `SynthesizerNode` trait + `NoRawAttestation` marker + `laws::` | 14 |
| `mapping` | `Mapping` trait (ordered key/value) + blanket `Vec<(K,V)>` impl + laws | 15 |
| `sequence` | `Sequence` trait (ordered) + blanket `Vec<T>` impl + laws | 12 |
| `indent` | `IndentStyle` (2sp / 4sp / tab) + `prefix` / `indent_line` / `indent_block` | 15 |
| `artifact` | `Artifact` trait + hierarchy (File, Workspace, Repo, Flake, Package, Gem, ContainerImage, HelmChart) + substrate parity + laws | 45+ |

Run: `cargo test` at the crate root.

## Artifact hierarchy

| Type | Kind | Notes |
|------|------|-------|
| `FileArtifact` | `File` | The atom: `(path, bytes)` |
| `Workspace` | `Workspace` | Directory pack (e.g., `platform-iam/`) |
| `Repo` | `Repo` | Git repository |
| `Flake` | `Flake` | Nix flake (repo with `flake.nix`) |
| `Package` | `Library`/`Gem`/`NpmPackage` | Versioned package in an [`Ecosystem`] |
| `Gem` | `Gem` | Ruby gem (specialized Package — pangea-* gems are central) |
| `ContainerImage` | `DockerImage` | OCI manifest + layers |
| `HelmChart` | `HelmChart` | Chart.yaml + values + templates |

### `ArtifactKind` taxonomy

Mirrors substrate's `artifactKind` from `substrate/lib/types/foundation.nix`
(first 11 variants — `is_substrate_canonical()` returns true) plus pleme-io
typescape extensions:

- **Substrate parity:** Binary, Library, Service, DockerImage, WasmComponent,
  WasiService, HelmChart, NpmPackage, Gem, Scaffold, Overlay.
- **Pleme-io extensions:** File, Workspace, Repo, Flake, Constellation,
  Fleet, Ami, StorePath, TerraformModule, TerraformProvider.

Adding a new variant is a coordinated change with substrate — do not diverge.

## Traits at a glance

```rust
pub trait SynthesizerNode {
    fn emit(&self, indent: usize) -> String;
    fn indent_unit() -> &'static str;
    fn variant_id(&self) -> u8;
}

pub trait Artifact {
    fn name(&self) -> &str;
    fn kind(&self) -> ArtifactKind;
    fn files(&self) -> Vec<FileArtifact>;
    fn canonical_bytes(&self) -> Vec<u8> { canonical_encode(&self.files()) }
}

pub trait Mapping { /* ordered key/value — impls for Vec<(K,V)> */ }
pub trait Sequence { /* ordered — impls for Vec<T> */ }
```

## Content-addressing

`canonical_encode(&[FileArtifact]) -> Vec<u8>` is deterministic and
order-independent over path-unique sets:

- Sort by path.
- For each file: `len(path_bytes) ++ path_bytes ++ len(content) ++ content`.
- Little-endian u64 length prefixes.

Hash the output with BLAKE3 / SHA-256 to get a content-address. Two runs
of the same typescape state produce byte-identical canonical encodings —
divergence is a proof failure, not ambient variation.

## Wave 2 integration points

When real synthesizers adopt the traits:

- `impl SynthesizerNode for NixNode { ... }` — `indent_unit()` returns `"  "`, `variant_id` tags every variant.
- `impl Mapping for NixNode::AttrSet` — wrap underlying `Vec<Binding>`.
- `impl Sequence for NixNode::List` — trivial wrapper.
- Workspace builders return `Workspace` / `Flake` / `Repo` directly instead of `Vec<(PathBuf, String)>`.
- Each crate gains a test file calling `synthesizer_core::*::laws::*` — compounding proof surface.

## No-raw invariant

`NoRawAttestation::attestation()` is a short hand-written note per implementor
describing how no-raw is enforced (grep check, deprecation-and-delete plan,
etc.). Combined with a variant-coverage property test (every `variant_id`
exercised), a hidden Raw variant would cause test failures.

## Convergence lens

This crate is the vocabulary layer of the typescape's rendering dimension.
Every synthesizer projects its domain into source code; this trait set is
the minimum shared contract that lets proofs transfer across synthesizers
— declare the node, prove the laws, render anywhere.
