use yew::{function_component, html, Children, Html, Properties};

#[derive(PartialEq, Properties)]
pub struct PortWrapProps {
    pub children: Children,
}
#[function_component(PortWrap)]
pub fn port_wrap(PortWrapProps { children }: &PortWrapProps) -> Html {
    html! {
        <div class={"port-wrap"}>
            {for children.iter()}
        </div>
    }
}
