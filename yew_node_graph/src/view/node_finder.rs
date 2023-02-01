use crate::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct FinderProps {
    #[prop_or_default]
    pub children: Children,
    pub is_showing: bool,
    pub pos: Vec2,
}
#[styled_component(Finder)]
pub fn finder(
    FinderProps {
        children,
        is_showing,
        pos,
    }: &FinderProps,
) -> Html {
    let style = format!(
        "
display: {};
left: {}px;
top:{}px;
position:fixed;
    ",
        if *is_showing { "block" } else { "none" },
        pos.x,
        pos.y
    );
    html! {
        <div class={"node_finder"} {style}>
            { for children.iter() }
        </div>
    }
}
