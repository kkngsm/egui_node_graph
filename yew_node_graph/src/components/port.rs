use std::fmt::Display;

use glam::Vec2;
use yew::{
    function_component, html, use_effect_with_deps, use_node_ref, Callback, Html, Properties,
};

use crate::{state::AnyParameterId, utils::get_center};
#[derive(Properties, PartialEq)]
pub struct PortProps<PortId, DataType>
where
    PortId: PartialEq + Copy,
    DataType: PartialEq,
{
    pub typ: DataType,
    pub id: PortId,
    pub onevent: Callback<PortEvent>,
}
#[function_component(Port)]
pub fn port<PortId, DataType>(PortProps { typ, id, onevent }: &PortProps<PortId, DataType>) -> Html
where
    DataType: Display + PartialEq,
    PortId: Into<AnyParameterId> + PartialEq + Copy + 'static,
{
    let id = *id;
    let node_ref = use_node_ref();
    let onevent = onevent.clone();
    use_effect_with_deps(
        move |node_ref| {
            let element = node_ref.cast::<web_sys::Element>().unwrap();
            let global_pos = get_center(&element);
            onevent.emit(PortEvent::Rendered {
                id: id.into(),
                global_pos,
            })
        },
        node_ref.clone(),
    );
    html! {
        <div
            ref={node_ref}
            class={"port"}
            data-type={typ.to_string()}
        />
    }
}
#[derive(Debug, Clone)]
pub enum PortEvent {
    Rendered {
        id: AnyParameterId,
        global_pos: Vec2,
    },
}
