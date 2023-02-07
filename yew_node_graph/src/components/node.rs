use crate::{
    utils::{get_offset_from_target, on_event},
    Vec2,
};
use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct NodeProps {
    pub title: String,
    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,
    pub onevent: Option<Callback<NodeEvent>>,
}

/// Node component
/// if this node is selected, its html attribute `data-is-selected` is `true`
/// this components have `node` class
///
/// # Default style
/// ```css
/// position:absolute;
/// user-select:none;
/// display:inline-block;
/// ```
#[styled_component(Node)]
pub fn node(
    NodeProps {
        title,
        onevent,
        pos,
        is_selected,
    }: &NodeProps,
) -> Html {
    let node = css! {r#"
position:absolute;
user-select:none;
display:inline-block;
"#};
    html! {
        <div
            class={classes![
                node,
                "node"
            ]}
            style={format!("left:{}px;top:{}px;", pos.x, pos.y)}
            data-is-selected={is_selected.to_string()}
            onclick={on_event(onevent.clone(), |e| {
                e.stop_propagation();
                NodeEvent::Select{shift_key: e.shift_key()}
            })}
            onmousedown = {on_event(onevent.clone(), |e: MouseEvent| {
                NodeEvent::DragStart{gap: get_offset_from_target(&e), shift_key: e.shift_key()}
            })}
        >
            <div>{title}</div>
        </div>
    }
}

#[derive(Debug, Clone)]
pub enum NodeEvent {
    DragStart { gap: Vec2, shift_key: bool },
    Select { shift_key: bool },
}
