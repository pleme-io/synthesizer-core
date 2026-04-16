//! # synthesizer-core
//!
//! Shared typed-AST primitives and the provable artifact hierarchy for the
//! pleme-io synthesizer family.
//!
//! > "The big deal here is we render artifacts provably and reliably
//! > through typescape and AST."
//!
//! The central claim of this crate is a chain:
//!
//! ```text
//!     Rust types  →  typed AST nodes  →  bytes  →  Artifact  →  disk / registry / cluster
//! ```
//!
//! Every link is deterministic. Every link is under tests. Compose them and
//! every deliverable produced by pleme-io (a file, a workspace, a gem, an
//! OCI image, a Helm chart, a Terraform provider, an AMI) is content-
//! addressable by construction.
//!
//! ## This crate introduces no breaking changes
//!
//! Existing synthesizers keep working unchanged. The traits and types here
//! are opt-in: wave-by-wave, real synthesizers will implement
//! [`SynthesizerNode`] and return concrete [`Artifact`] types from their
//! workspace builders.
//!
//! ## Design principles
//!
//! - **Zero runtime dependencies.** Only `proptest` at dev-time.
//! - **Every abstraction ships with laws** in a `laws::` submodule so real
//!   synthesizers can call them from property tests to compound proof surface.
//! - **Nothing silently downgrades concrete knowledge.** Adopting the traits
//!   is additive; no existing test is weakened.
//! - **Taxonomy parity with substrate.** [`ArtifactKind`] mirrors
//!   `substrate/lib/types/foundation.nix`'s `artifactKind` enum plus
//!   pleme-io typescape extensions.
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`node`] | [`SynthesizerNode`] trait + [`NoRawAttestation`] marker + laws |
//! | [`mapping`] | [`Mapping`] trait — ordered key/value (AttrSet, Hash, Map) |
//! | [`sequence`] | [`Sequence`] trait — ordered elements (List, Array, Seq) |
//! | [`indent`] | [`IndentStyle`] — parameterized indentation |
//! | [`artifact`] | [`Artifact`] trait + hierarchy (File, Workspace, Repo, Flake, Package, Gem, ContainerImage, HelmChart, …) |

pub mod artifact;
pub mod indent;
pub mod mapping;
pub mod node;
pub mod sequence;

pub use artifact::{
    canonical_encode, Artifact, ArtifactKind, ContainerImage, Ecosystem, FileArtifact, Flake,
    Gem, HelmChart, Package, Repo, Workspace,
};
pub use indent::IndentStyle;
pub use mapping::Mapping;
pub use node::{NoRawAttestation, SynthesizerNode};
pub use sequence::Sequence;
