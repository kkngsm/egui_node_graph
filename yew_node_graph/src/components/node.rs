use crate::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;

use crate::hooks::{use_drag, DragEvent};
#[derive(Properties, PartialEq)]
pub struct NodeProps {
    pub title: String,
    pub onevent: Option<Callback<NodeEvent>>,
    pub pos: Vec2,
}
#[styled_component(Node)]
pub fn node(
    NodeProps {
        title,
        onevent,
        pos,
    }: &NodeProps,
) -> Html {
    let div_ref = {
        let onevent = onevent.clone();
        use_drag(move |event| {
            let event = NodeEvent::Drag(event);
            if let Some(onevent) = onevent.as_ref() {
                onevent.emit(event);
            }
        })
    };
    let node = css! {r#"
position:relative;
user-select:none;
display:inline-block;
"#};
    html! {
        <div
            ref={div_ref}
            class={classes![
                node,
                "node",
                //  if is_dragging {"is_dragging"} else {"is_not_dragging"}
            ]}
            style={format!("left:{}px;top:{}px;", pos.x, pos.y)}
        >
            <div>{title}</div>
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeEvent {
    Drag(DragEvent),
}
