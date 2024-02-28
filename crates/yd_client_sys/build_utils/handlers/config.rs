use super::{handle_function_parameter::ParameterFlavor, handle_function_prototype::MethodFlavor};

#[derive(Clone)]
pub struct HandlerConfigs {
    pub method_flavor: MethodFlavor,
    pub parameter_flavor: ParameterFlavor,
    pub record_name: String,
}

impl Default for HandlerConfigs {
    fn default() -> Self {
        HandlerConfigs {
            method_flavor: MethodFlavor::None,
            parameter_flavor: ParameterFlavor::None,
            record_name: "DefaultName".to_string(),
        }
    }
}
