pub mod widget;
pub mod wrap;
use std::fmt::Display;
use web_sys::MouseEvent;
use yew::{
    function_component, html, Callback, Html, NodeRef,
    Properties,
};

use crate::state::AnyParameterId;
#[derive(Properties, PartialEq)]
pub struct PortProps<PortId, DataType>
where
    PortId: PartialEq + Copy,
    DataType: PartialEq,
{
    pub typ: DataType,
    pub id: PortId,
    pub is_should_draw: bool,
    pub node_ref: NodeRef,
    pub onevent: Callback<PortEvent>,
}
#[function_component(Port)]
pub fn port<PortId, DataType>(
    PortProps {
        typ,
        id,
        is_should_draw,
        node_ref,
        onevent,
    }: &PortProps<PortId, DataType>,
) -> Html
where
    DataType: Display + PartialEq,
    PortId: Into<AnyParameterId> + PartialEq + Copy + 'static,
{
    let id = *id;
    let is_should_draw = *is_should_draw;
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
}
