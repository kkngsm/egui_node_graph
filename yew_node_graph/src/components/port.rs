use std::fmt::Display;

use glam::Vec2;
use web_sys::MouseEvent;
use yew::{
    function_component, html, use_effect_with_deps, use_node_ref, Callback, Html, NodeRef,
    Properties,
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
    pub is_should_draw: bool,
    pub onevent: Callback<PortEvent>,
}
#[function_component(Port)]
pub fn port<PortId, DataType>(
    PortProps {
        typ,
        id,
        is_should_draw,
        onevent,
    }: &PortProps<PortId, DataType>,
) -> Html
where
    DataType: Display + PartialEq,
    PortId: Into<AnyParameterId> + PartialEq + Copy + 'static,
{
    let id = *id;
    let node_ref = use_node_ref();
    let is_should_draw = *is_should_draw;
    use_effect_with_deps(
        {
            let onevent = onevent.clone();
            move |node_ref: &NodeRef| {
                let element = node_ref.cast::<web_sys::Element>().unwrap();
                let global_pos = get_center(&element);
                onevent.emit(PortEvent::Rendered {
                    id: id.into(),
                    global_pos,
                })
            }
        },
        node_ref.clone(),
    );
    html! {
        <div
            onmousedown={{
                let onevent = onevent.clone();
                move|e:MouseEvent| if is_should_draw{
                        e.stop_propagation();
                        onevent.emit(PortEvent::MouseDown(id.into()))
                }
            }}
            ref={node_ref}
            class={"port"}
            data-type={typ.to_string()}
            data-is-should-draw={is_should_draw.to_string()}
        />
    }
}
#[derive(Debug, Clone)]
pub enum PortEvent {
    MouseDown(AnyParameterId),
    Rendered {
        id: AnyParameterId,
        global_pos: Vec2,
    },
}
