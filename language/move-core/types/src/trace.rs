use crate::identifier::Identifier;
use crate::language_storage::ModuleId;
pub use crate::language_storage::TypeTag;
use crate::{value::MoveValue, vm_status::VMStatus};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallType {
    Call,
    CallGeneric,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTrace {
    pub depth: u32,
    pub call_type: CallType,
    pub module_id: Option<ModuleId>,
    pub function: Identifier,
    pub ty_args: Vec<TypeTag>,
    pub args: Vec<MoveValue>,
    pub gas_used: u64,
    pub err: Option<VMStatus>,
}
