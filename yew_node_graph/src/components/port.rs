pub mod unit;
pub mod widget;
use std::fmt::Display;
use web_sys::MouseEvent;
use yew::{function_component, html, Callback, Html, NodeRef, Properties};

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
    pub onevent: Callback<(AnyParameterId, PortEvent)>,
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
                move|e:MouseEvent| {
                        e.stop_propagation();
                        onevent.emit((id.into(), PortEvent::MouseDown))
                }
            }}
            onmouseenter={{
                let onevent = onevent.clone();
                move|_:MouseEvent| {
                        onevent.emit((id.into(), PortEvent::MouseEnter))
                }
            }}
            onmouseleave={{
                let onevent = onevent.clone();
                move|_:MouseEvent| {
                    onevent.emit((id.into(), PortEvent::MouseLeave))
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
    MouseDown,
    MouseEnter,
    MouseLeave,
}
