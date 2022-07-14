use std::any::TypeId;

use thiserror::Error;

use super::ArchetypeType;


#[derive(Debug, Error)]
pub enum BuildEntityError {
    #[error("Expected archetype_type: [{expected:?}], found archetype_type: [{actual:?}]")]
    MismatchedArchetype {expected: ArchetypeType, actual: ArchetypeType}
}

pub type BuildEntityResult<T> = Result<T, BuildEntityError>;

#[derive(Debug, Error)]
pub enum AddSystemError {
    #[error("System exist: [{system_name:?}] [{system_type_id:?}]")]
    SystemExist { system_name: String, system_type_id: TypeId }
}

pub type AddSystemResult<T> = Result<T, AddSystemError>;