mod config;
pub mod decompiler;
pub mod detectors;
pub mod graph;
pub mod provider;
pub mod sierra_program;
pub mod sym_exec;

use std::fs;
use std::path::Path;
use std::process::exit;

use cairo_lang_starknet_classes::contract_class::ContractClass;
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::DetectorType;
use crate::detectors::get_detectors;
use crate::sierra_program::SierraProgram;

/// Load the Sierra program from the /target directory
async fn load_scarb_program() -> Result<SierraProgram, String> {
    let target_dir = Path::new("./target/dev/");

    // Read the directory contents
    let entries =
        fs::read_dir(target_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    // Find the file that ends with "contract_class.json"
    let contract_class_file = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| name.ends_with("contract_class.json"))
            {
                Some(path)
            } else {
                None
            }
        })
        .next();

    // Check if the file was found
    let contract_class_file = if let Some(file) = contract_class_file {
        file
    } else {
        eprintln!("You need to run scarb build before running the sierra-analyzer");
        exit(1);
    };

    // Open the file
    let mut file = File::open(&contract_class_file)
        .await
        .map_err(|e| format!("Failed to open file: {}", e))?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Deserialize the JSON content into a ContractClass
    let contract_class: Result<ContractClass, _> = serde_json::from_str(&content);

    let program_string = match contract_class {
        Ok(ref prog) => {
            // Extract the Sierra program from the ContractClass
            match prog.extract_sierra_program() {
                Ok(prog_sierra) => prog_sierra.to_string(),
                Err(e) => {
                    eprintln!("Error extracting Sierra program: {}", e);
                    content.clone()
                }
            }
        }
        Err(ref _e) => content.clone(),
    };

    // Initialize a new SierraProgram with the deserialized Sierra program content
    let mut program = SierraProgram::new(program_string);

    // Set the program ABI if deserialization was successful
    if let Ok(ref contract_class) = contract_class {
        let abi = contract_class.abi.clone();
        program.set_abi(abi.unwrap());
    }

    Ok(program)
}

/// Handle the running of detectors and printing their results
fn handle_detectors(decompiler: &mut Decompiler, detector_names: Vec<String>) {
    let mut detectors = get_detectors();
    let mut output = String::new();

    // Run the specified detectors
    for detector in detectors.iter_mut() {
        // Skip TESTING detectors if no specific detector names are provided
        if detector_names.is_empty() && detector.detector_type() == DetectorType::TESTING {
            continue;
        }

        // Skip detectors not in the provided names if names are provided
        if !detector_names.is_empty() && !detector_names.contains(&detector.id().to_string()) {
            continue;
        }

        let result = detector.detect(decompiler);
        if !result.trim().is_empty() {
            // Each detector output is formatted like
            //
            // [Detector category] Detector name
            //      - detector content
            //      - ...
            output.push_str(&format!(
                "[{}] {}\n{}\n\n",
                detector.detector_type().as_str(),
                detector.name(),
                result
                    .lines()
                    .map(|line| format!("\t- {}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }
    }

    // Print the detectors result if not empty
    if !output.trim().is_empty() {
        println!("{}", output.trim());
    }
}

pub async fn analyze_project() {
    // Load the Sierra program from the /target directory
    let program = match load_scarb_program().await {
        Ok(program) => program,
        Err(_e) => {
            eprintln!("Error loading program, you must build it before running the analyzer");
            return;
        }
    };

    // Initialize the decompiler
    let mut decompiler = program.decompiler(false);
    let decompiled_code = decompiler.decompile(false);

    // Run the detectors
    handle_detectors(&mut decompiler, Vec::new());

    // Print the decompiled code
    println!("{}", decompiled_code);
}
