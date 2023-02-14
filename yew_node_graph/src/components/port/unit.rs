use yew::{function_component, html, Children, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct PortUnitProps {
    pub children: Children,
}
#[function_component(PortUnit)]
pub fn port_unit(PortUnitProps { children }: &PortUnitProps) -> Html {
    html! {
        <div class={"port-unit"}>
            {for children.iter()}
        </div>
    }
}
