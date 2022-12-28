use crate::{
    value::{MoveTypeLayout, MoveValue},
    vm_status::VMStatus,
};

#[derive(Clone, Debug)]
pub enum CallType {
    Call,
    CallGeneric,
}

#[derive(Clone, Debug)]
pub struct CallTrace {
    pub depth: u32,
    pub call_type: CallType,
    pub module_id: Option<String>,
    pub function: String,
    pub ty_args: Vec<MoveTypeLayout>,
    pub args: Vec<MoveValue>,
    pub gas_used: u64,
    pub err: Option<VMStatus>,
}
