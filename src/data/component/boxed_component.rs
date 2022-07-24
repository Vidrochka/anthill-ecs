// use std::any::Any;

// use crate::types::ComponentId;

// pub struct BoxedComponent {
//     pub (crate) data: Box<dyn Any + Sync + Send>,
//     pub (crate) component_id: ComponentId,
// }

// impl Into<BoxedComponent> for Box<dyn Any + Send + Sync> {
//     fn into(self) -> BoxedComponent {
//         BoxedComponent {
//             data: self,
//             component_id: ComponentId::new((*self).type_id()),
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use std::any::{Any, TypeId};


//     struct TestComponent {

//     }

//     fn check_into(components: Vec<Box<dyn Any + Sync + Send>>) {
//         let t: super::BoxedComponent = components[0].into();
//         assert_eq!(*t.component_id, TypeId::of::<TestComponent>()); 
//     }

//     #[test]
//     fn test_boxed_component() {
//         check_into(vec![Box::new(TestComponent {})])
//     }

// }