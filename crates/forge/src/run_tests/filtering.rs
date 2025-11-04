use crate::{
    run_tests::workspace::PackagesWithTestTargets,
    test_filter::{NameFilter, TestsFilter},
};
use forge_runner::running::with_config::{TestCandidate, TestTarget};
use std::collections::HashMap;

pub enum FilterResult {
    Included(TestCandidate),
    Excluded,
}

pub fn apply_name_filter(
    packages_with_test_targets: PackagesWithTestTargets<TestCandidate>,
    tests_filter: &TestsFilter,
) -> PackagesWithTestTargets<FilterResult> {
    let mut result = PackagesWithTestTargets(HashMap::new());

    for (package_id, test_targets) in packages_with_test_targets.0 {
        let filtered_test_targets = test_targets
            .into_iter()
            .map(|test_target| {
                let tests = test_target
                    .tests
                    .into_iter()
                    .map(|test_candidate| match tests_filter.name_filter {
                        NameFilter::All => FilterResult::Included(test_candidate),
                        NameFilter::Match(ref name) => {
                            if test_candidate.name.contains(name) {
                                FilterResult::Included(test_candidate)
                            } else {
                                FilterResult::Excluded
                            }
                        }
                        NameFilter::ExactMatch(ref name) => {
                            if test_candidate.name == *name {
                                FilterResult::Included(test_candidate)
                            } else {
                                FilterResult::Excluded
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                TestTarget {
                    tests_location: test_target.tests_location,
                    sierra_program: test_target.sierra_program,
                    sierra_program_path: test_target.sierra_program_path,
                    casm_program: test_target.casm_program,
                    tests,
                }
            })
            .collect::<Vec<_>>();

        result.0.insert(package_id, filtered_test_targets);
    }

    result
}
