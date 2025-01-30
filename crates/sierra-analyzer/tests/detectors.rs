use sierra_analyzer::detectors::detector::Detector;
use sierra_analyzer::detectors::functions_detector::FunctionsDetector;
use sierra_analyzer::detectors::statistics_detector::StatisticsDetector;
use sierra_analyzer::detectors::strings_detector::StringsDetector;
use sierra_analyzer::detectors::felt_overflow_detector::FeltOverflowDetector;
use sierra_analyzer::sierra_program::SierraProgram;

#[test]
fn test_string_detector() {
    // Read file content
    let content = include_str!("../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the strings detector
    let mut detector = StringsDetector::new();

    // Detected strings
    let detected_strings = detector.detect(&mut decompiler);

    let expected_output = r#"Index out of bounds
u32_sub Overflow"#;

    assert_eq!(detected_strings, expected_output);
}

#[test]
fn test_functions_detector() {
    // Read file content
    let content = include_str!("../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the functions name detector
    let mut detector = FunctionsDetector::new();

    // functions names
    let functions_names = detector.detect(&mut decompiler);

    let expected_output =
        "Private : examples::fib_array::fib\n\u{1b}[0mPrivate : examples::fib_array::fib_inner\u{1b}[0m";

    assert_eq!(functions_names, expected_output);
}

#[test]
fn test_statistics_detector() {
    // Read file content
    let content = include_str!("../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the statistics detector
    let mut detector = StatisticsDetector::new();

    // Program statistics
    let statistics = detector.detect(&mut decompiler);

    let expected_output = r#"Libfuncs: 42
Types: 19
Functions: 2"#;

    assert_eq!(statistics, expected_output);
}

#[test]
fn test_integer_overflow_detector() {
    // Read file content
    let content =
        include_str!("../examples/sierra/test_contract__test_contract.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the FeltOverflow detector
    let mut detector = FeltOverflowDetector::new();

    // Detected felt overflows
    let statistics = detector.detect(&mut decompiler);

    let expected_output = r#"cairo_level_tests::contracts::test_contract::test_contract::__wrapper__Impl__test: method felt252_add could be used to trigger a felt overflow/underflow (Confidence: Low)
cairo_level_tests::contracts::test_contract::test_contract::__wrapper__Impl__test: method felt252_add could be used to trigger a felt overflow/underflow (Confidence: Low)
"#;

    assert_eq!(statistics, expected_output);
}