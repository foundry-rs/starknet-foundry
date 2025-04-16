use cairo_lang_casm::hints::Hint;
use cairo_vm::serde::deserialize_program::{ApTracking, FlowTrackingData, HintParams};
use std::collections::HashMap;
use universal_sierra_compiler_api::AssembledCairoProgramWithSerde;

pub fn hints_by_representation(
    assembled_program: &AssembledCairoProgramWithSerde,
) -> HashMap<String, Hint> {
    assembled_program
        .hints
        .iter()
        .flat_map(|(_, hints)| hints.iter().cloned())
        .map(|hint| (hint.representing_string(), hint))
        .collect()
}

#[must_use]
pub fn hints_to_params(
    assembled_program: &AssembledCairoProgramWithSerde,
) -> HashMap<usize, Vec<HintParams>> {
    assembled_program
        .hints
        .iter()
        .map(|(offset, hints)| {
            (
                *offset,
                hints
                    .iter()
                    .map(|hint| HintParams {
                        code: hint.representing_string(),
                        accessible_scopes: vec![],
                        flow_tracking_data: FlowTrackingData {
                            ap_tracking: ApTracking::new(),
                            reference_ids: HashMap::new(),
                        },
                    })
                    .collect::<Vec<HintParams>>(),
            )
        })
        .collect()
}
