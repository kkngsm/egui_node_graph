use crate::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;

/// Properties of [`ContextMenu`]
#[derive(Properties, PartialEq)]
pub struct ContextMenuProps {
    #[prop_or_default]
    pub children: Children,
    /// display: `block` or `none`
    pub is_showing: bool,
    /// Top left position of context menu
    pub pos: Vec2,
}

/// ContextMenu component
///
/// The following are the HTML attributes of this component.
/// The minimum style that does not interfere with operation is set.
/// ```text
/// class: "edge"
/// data-type: `DataType::to_string()`
/// style: {
///     position:absolute;
///     top: {}px;
///     left: {}px;
///     display: {};
/// }
/// ```
#[styled_component(ContextMenu)]
pub fn contextmenu(
    ContextMenuProps {
        children,
        is_showing,
        pos,
    }: &ContextMenuProps,
) -> Html {
    let style = format!(
        "
display: {};
left: {}px;
top:{}px;
position:absolute;
    ",
        if *is_showing { "block" } else { "none" },
        pos.x,
        pos.y
    );
    html! {
        <div class={"node-graph-contextmenu"} {style}>
            { for children.iter() }
        </div>
    }
}
