use super::{
    handle_function_parameter::ParameterFlavor, handle_function_prototype::MethodFlavor,
    handle_record::RecordFlavor,
};

#[derive(Clone)]
pub struct HandlerConfigs {
    pub method_flavor: MethodFlavor,
    pub parameter_flavor: ParameterFlavor,
    pub record_flavor: RecordFlavor,
    pub record_name: String,
    /// the index of this child in its parent
    pub index: usize,
    /// how many children does the parent have, only count children with same type (and processed by same handler)
    pub num_parent_children_same_handler: usize,
}

impl Default for HandlerConfigs {
    fn default() -> Self {
        HandlerConfigs {
            method_flavor: MethodFlavor::None,
            parameter_flavor: ParameterFlavor::None,
            record_flavor: RecordFlavor::None,
            record_name: "WarningRecordNameUnfilled".to_string(),
            index: 0,
            num_parent_children_same_handler: 0,
        }
    }
}
