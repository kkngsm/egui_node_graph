use glam::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;

use crate::utils::{get_offset_from_current_target, on_event, use_event_listeners};
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub children: Children,
    pub onevent: Option<Callback<BackgroundEvent>>,
}
#[styled_component(GraphArea)]
pub fn graph_area(GraphProps { children, onevent }: &GraphProps) -> Html {
    let node_ref = use_event_listeners([
        (
            "contextmenu",
            Box::new(on_event(onevent.clone(), |e| {
                e.prevent_default();
                BackgroundEvent::ContextMenu(get_offset_from_current_target(e))
            })),
        ),
        (
            "click",
            Box::new(on_event(onevent.clone(), |e| {
                BackgroundEvent::Click(get_offset_from_current_target(e))
            })),
        ),
        // TODO: It would be better to add it to the Document,
        // but I don't know how to get the relative position of the mouse coordinates,
        // so I'll do it later.
        (
            "mousemove",
            Box::new(on_event(onevent.clone(), |e| {
                BackgroundEvent::Move(get_offset_from_current_target(e))
            })),
        ),
    ]);
    let graph_area = css!(
        r#"
position:relative;
"#
    );
    html! {
        <div
            ref={node_ref}
            class={classes![graph_area,"graph-area"]}

        >
            { for children.iter() }
        </div>
    }
}

#[derive(Debug)]
pub enum BackgroundEvent {
    ContextMenu(Vec2),
    Click(Vec2),
    Move(Vec2),
    MouseUp(Vec2),
}
