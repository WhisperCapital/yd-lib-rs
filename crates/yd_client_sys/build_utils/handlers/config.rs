use super::{
    handle_function_parameter::ParameterFlavor, handle_function_prototype::MethodFlavor,
    handle_record::RecordFlavor,
};

#[derive(Clone, Debug)]
pub struct HandlerConfigs {
    pub method_flavor: MethodFlavor,
    pub parameter_flavor: ParameterFlavor,
    pub record_flavor: RecordFlavor,
    pub record_name: String,
    pub life_time: String,
    /// modified by children, to let parent know if any child use a life time parameter
    pub life_time_on_children: bool,
    pub prefer_pointer: bool,
    /// the index of this child in its parent
    pub index: usize,
    /// how many children does the parent have, only count children with same type (and processed by same handler)
    pub num_parent_children_same_handler: usize,
    pub debug: bool,
}

impl Default for HandlerConfigs {
    fn default() -> Self {
        HandlerConfigs {
            method_flavor: MethodFlavor::None,
            parameter_flavor: ParameterFlavor::None,
            record_flavor: RecordFlavor::None,
            life_time: "".to_string(),
            life_time_on_children: false,
            prefer_pointer: false,
            record_name: "WarningRecordNameUnfilled".to_string(),
            index: 0,
            num_parent_children_same_handler: 0,
            debug: false,
        }
    }
}
