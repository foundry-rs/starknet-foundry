use cairo_lang_sierra::extensions::core::CoreLibfunc;
use cairo_lang_sierra::extensions::core::CoreType;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra::ProgramParser;
use cairo_lang_starknet_classes::abi::Contract;

use crate::decompiler::decompiler::Decompiler;

/// A struct that represents a Sierra program
pub struct SierraProgram {
    /// The parsed Sierra program
    program: Program,

    /// Program registry
    registry: ProgramRegistry<CoreType, CoreLibfunc>,

    /// Contract ABI
    pub abi: Option<Contract>,
}

impl SierraProgram {
    /// Creates a new `SierraProgram` instance by parsing the given Sierra code
    pub fn new(content: String) -> Self {
        let program = match ProgramParser::new().parse(&content) {
            Ok(program) => program,
            Err(err) => {
                panic!("Error parsing Sierra code: {}", err);
            }
        };

        let registry = match ProgramRegistry::<CoreType, CoreLibfunc>::new(&program) {
            Ok(registry) => registry,
            Err(err) => {
                panic!("Error creating program registry: {}", err);
            }
        };

        SierraProgram {
            program,
            registry,
            abi: None,
        }
    }

    /// Returns a reference to the parsed Sierra program
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Returns a reference to the program registry
    pub fn registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
        &self.registry
    }

    /// Decompiles the Sierra program and returns a `Decompiler` instance
    pub fn decompiler(&self, verbose: bool) -> Decompiler {
        Decompiler::new(self, verbose)
    }

    /// Sets the ABI of the contract
    pub fn set_abi(&mut self, abi: Contract) {
        self.abi = Some(abi);
    }
}
