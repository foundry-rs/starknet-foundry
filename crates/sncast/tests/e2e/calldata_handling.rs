#[test]
fn test_no_arguments_with_required_calldata() {
    // Setup test contract that requires calldata
    let contract = deploy_contract_requiring_calldata();
    
    // Call without --arguments or --calldata
    let result = sncast()
        .args(&["call", "--contract", &contract.address, "--function", "requires_args"])
        .assert()
        .failure();
        
    // Verify we get data transformer error, not provider error
    assert!(result.stderr().contains("Failed to serialize arguments"));
}

#[test]
fn test_both_arguments_and_calldata_provided() {
    let contract = deploy_contract_requiring_calldata();
    
    let result = sncast()
        .args(&[
            "call",
            "--contract", &contract.address,
            "--function", "requires_args",
            "--calldata", "1",
            "--arguments", "2"
        ])
        .assert()
        .failure();
        
    assert!(result.stderr().contains("Cannot provide both --calldata and --arguments"));
}

#[test]
fn test_invalid_calldata_format() {
    let contract = deploy_contract_requiring_calldata();
    
    let result = sncast()
        .args(&[
            "call",
            "--contract", &contract.address,
            "--function", "requires_args",
            "--calldata", "invalid"
        ])
        .assert()
        .failure();
        
    assert!(result.stderr().contains("Failed to parse calldata value to felt"));
} 