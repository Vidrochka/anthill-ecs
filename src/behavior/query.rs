use std::collections::HashSet;

use crate::{types::{ComponentId, ArchetypeType}, data::archetype::ArchetypeChunk};


pub struct ArchetypeQuery {
    pub (crate) required: Option<HashSet<ComponentId>>,
    pub (crate) except: Option<HashSet<ComponentId>>,
    pub (crate) addition: Option<HashSet<ComponentId>>,
    pub (crate) updated: Option<u32>,
}

impl ArchetypeQuery {
    pub fn new(required: Option<HashSet<ComponentId>>, except: Option<HashSet<ComponentId>>, addition: Option<HashSet<ComponentId>>, updated: Option<u32>) -> Self {
        Self {
            required,
            except,
            addition,
            updated
        }
    }

    pub fn is_archetype_match(&self, archetype_type: &ArchetypeType) -> bool {
        if let Some(required) = &self.required {
            let all_required_exist = required.iter().all(|x| archetype_type.contains(x));

            if !all_required_exist {
                return false;
            }
        }

        if let Some(except) = &self.except {
            let any_except_exist = except.iter().any(|x| archetype_type.contains(x));

            if any_except_exist {
                return false;
            }
        }

        return true;
    }

    pub fn is_chunk_match(&self, archetype_chunk: &ArchetypeChunk) -> bool {
        //add check updated
        true
    }
}