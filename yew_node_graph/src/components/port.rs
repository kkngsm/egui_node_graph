use std::fmt::Display;

use crate::state::AnyParameterId;
use yew::{function_component, html, Callback, Html, Properties};
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
    html! {
        <div
            class={"port"}
            data-type={typ.to_string()}
        />
    }
}
#[derive(Debug, Clone)]
pub enum PortEvent {}
