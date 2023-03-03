use yew::{function_component, html, Children, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct PortUnitProps {
    pub children: Children,
}

/// This Component have Port and Port Widget.
///
/// This is used to combine ports and PortWidget into one when multiple ports are lined up.
///
/// The following are the HTML attributes of this component.
/// ```text
/// class: "port-unit"
/// ```
#[function_component(PortUnit)]
pub fn port_unit(PortUnitProps { children }: &PortUnitProps) -> Html {
    html! {
        <div class={"port-unit"}>
            {for children.iter()}
        </div>
    }
}
