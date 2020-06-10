//! Defines the `PE` security mitigation checker. Consumes an
//! PE binary, parses it, and checks for the following features:
//!
//! * NX (Non-eXecutable bit)
//! * Full/Partial RELRO
//! * Position-Independent Executable / ASLR

use goblin::pe::PE;

use crate::check::{BinFeatures, BinInfo, Checker};

/// struct defining security features parsed from ELF, and
/// derives serde de/serialize traits for structured output.
/// TODO
pub struct PEChecker {
    pub authenticode: bool,
    pub dep: bool,
}
