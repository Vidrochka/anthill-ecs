// use std::{collections::HashMap, any::Any};
// use std::fmt::Debug;

// use crate::types::{ComponentId, ArchetypeType};

// use super::archetype::{IComponentsArray, ComponentsArray, ArchetypeChunk, ArchetypeChunkFabricClosure};

// pub struct NewEntityComponentsInfo {
//     pub components_map: HashMap<ComponentId, Box<dyn Any + Send + Sync>>,
//     pub archetype_chunk_fabric_closure: Box<dyn ArchetypeChunkFabricClosure + Sync + Send>,
// }

// pub struct


// // impl<T0, T1> Into<ArchetypeInfo> for (T0, T1)
// // where
// //     T0: Sync + Send + 'static,
// //     T1: Sync + Send + 'static,
// // {
// //     fn into(self) -> ArchetypeInfo {
// //         let components_map = HashMap::new();
// //         components_map.insert(ComponentId::from_type::<T0>(), Box::new(self.0) as Box<dyn Any + Send + Sync>);
// //         components_map.insert(ComponentId::from_type::<T1>(), Box::new(self.1) as Box<dyn Any + Send + Sync>);
// //         let archetype_chunk_fabric_closure = Box::new(move || -> _ {
// //             let archetype_components_map = HashMap::new();
// //             archetype_components_map.insert(ComponentId::from_type::<T0>(), Box::new(ComponentsArray::<T0>::new()) as Box<dyn IComponentsArray>);
// //             archetype_components_map.insert(ComponentId::from_type::<T1>(), Box::new(ComponentsArray::<T1>::new()) as Box<dyn IComponentsArray>);

// //             ArchetypeChunk::new(archetype_components_map)
// //         });
// //         ArchetypeInfo {
// //             components_map,
// //             archetype_chunk_fabric_closure
// //         }
// //     }
// // }

// macro_rules! component_tuple_into_new_entity_components_info {
//     ( $( $name:ident ),+ ) => {
//         impl<$($name: Debug + Sync + Send + 'static),+> Into<NewEntityComponentsInfo> for ($($name,)+)
//         {
//             fn into(self) -> NewEntityComponentsInfo {
//                 let mut components_map = HashMap::new();
//                 let ($($name,)+) = self;
//                 $(components_map.insert(ComponentId::from_type::<$name>(), Box::new($name) as Box<dyn Any + Send + Sync>);)+
                
//                 let archetype_chunk_fabric_closure = Box::new(move || -> _ {
//                     let mut archetype_components_map = HashMap::new();

//                     $(archetype_components_map.insert(ComponentId::from_type::<$name>(), Box::new(ComponentsArray::<$name>::new()) as Box<dyn IComponentsArray>);)+
        
//                     ArchetypeChunk::new(archetype_components_map)
//                 });

//                 NewEntityComponentsInfo {
//                     components_map,
//                     archetype_chunk_fabric_closure
//                 }
//             }
//         }
//     };
// }

// component_tuple_into_new_entity_components_info!(T0);
// component_tuple_into_new_entity_components_info!(T0, T1);
// component_tuple_into_new_entity_components_info!(T0, T1, T2);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5, T6);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5, T6, T7);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
// component_tuple_into_new_entity_components_info!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);