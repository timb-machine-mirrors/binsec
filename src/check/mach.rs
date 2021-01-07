//! Defines the `Mach` security mitigation checker. Consumes an
//! Mach-O binary, parses it, and checks for the following features:
//!
//! * NX (Non-eXecutable bit) stack
//! * NX (Non-eXecutable bit) heap
//! * Position-Independent Executable
//! * Stack Canaries
//! * Restricted segment

use goblin::mach::constants::cputype;
use goblin::mach::header;
use goblin::mach::MachO;

use serde::{Deserialize, Serialize};

use structmap::ToHashMap;
use structmap::value::Value;
use structmap_derive::ToHashMap;

use crate::check::Checker;
use crate::format::FeatureMap;

const MH_ALLOW_STACK_EXECUTION: u32 = 0x0002_0000;
const MH_NO_HEAP_EXECUTION: u32 = 0x0100_0000;

/// Struct defining parsed basic info from a Mach-O binary format
#[derive(Deserialize, Serialize, ToHashMap, Default)]
pub struct MachInfo {
    //#[rename("Machine")]
    pub machine: String,

    //#[rename("File Type")]
    pub filetype: String,

    //#[rename("Flags")]
    pub flags: String,

    //#[rename("Number of Load Commands")]
    pub num_cmds: usize,
}


/// Struct defining security features parsed from PE, and
/// derives serde de/serialize traits for structured output.
#[derive(Deserialize, Serialize, ToHashMap)]
pub struct MachChecker {
    // executable stack
    #[rename(name = "Non-Executable Stack")]
    pub nx_stack: bool,

    // executable heap
    #[rename(name = "Non-Executable Heap")]
    pub nx_heap: bool,

    // prevents out of bounds read/writes
    #[rename(name = "Stack Canary")]
    pub stack_canary: bool,

    // restricted segment for code injection prevention
    #[rename(name = "__RESTRICT")]
    pub restrict: bool,
}

impl Default for MachChecker {
    fn default() -> Self {
        Self {
            nx_stack: false,
            nx_heap: false,
            stack_canary: false,
            restrict: false,
        }
    }
}

impl Checker for MachO<'_> {
    fn bin_info(&self) -> FeatureMap {
        // parse out machine architecture given cpu type and subtype
        let machine: String =
            cputype::get_arch_name_from_types(self.header.cputype(), self.header.cpusubtype())
                .unwrap()
                .to_string();

        // parse out flag
        let flags: String = header::flag_to_str(self.header.flags).to_string();

        // parse out filetype
        let filetype: String = header::filetype_to_str(self.header.filetype).to_string();

        Box::new(MachInfo {
            machine,
            flags,
            filetype,
            num_cmds: self.header.ncmds,
        })
    }

    fn harden_check(&self) -> FeatureMap {
        // check for non-executable stack
        let nx_stack: bool = matches!(self.header.flags & MH_ALLOW_STACK_EXECUTION, 0);

        // check for non-executable heap
        let nx_heap: bool = matches!(self.header.flags & MH_NO_HEAP_EXECUTION, 0);

        // check for stack canary by finding canary functions in imports
        let stack_canary: bool = match self.imports() {
            Ok(imports) => imports
                .iter()
                .any(|x| x.name == "__stack_chk_fail" || x.name == "__stack_chk_guard"),
            Err(_) => false,
        };

        // check for __RESTRICT section for stopping dynlib injection
        let restrict: bool = self
            .segments
            .iter()
            .filter_map(|s| {
                if let Ok(name) = s.name() {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .any(|s| s.to_lowercase() == "__restrict");

        Box::new(MachChecker {
            nx_stack,
            nx_heap,
            stack_canary,
            restrict,
        })
    }
}
