use glam::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;

use crate::utils::{get_offset_from_current_target, use_event_listeners};
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub children: Children,
    pub node_ref: NodeRef,
    pub onevent: Callback<BackgroundEvent>,
}
#[styled_component(GraphArea)]
pub fn graph_area(
    GraphProps {
        children,
        onevent,
        node_ref,
    }: &GraphProps,
) -> Html {
    use_event_listeners(
        node_ref.clone(),
        [
            (
                "contextmenu",
                Box::new({
                    let onevent = onevent.clone();
                    move |e| {
                        e.prevent_default();
                        onevent.emit(BackgroundEvent::ContextMenu(
                            get_offset_from_current_target(&e),
                        ))
                    }
                }),
            ),
            (
                "click",
                Box::new({
                    let onevent = onevent.clone();
                    move |e| {
                        onevent.emit(BackgroundEvent::Click(get_offset_from_current_target(&e)))
                    }
                }),
            ),
        ],
    );
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
}
