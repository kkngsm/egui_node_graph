use glam::Vec2;
use stylist::yew::styled_component;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::utils::{get_mouse_pos_from_current_target, use_event_listeners};

/// Properties of [`GraphArea`]
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub children: Children,
    pub node_ref: NodeRef,
    pub onevent: Callback<BackgroundEvent>,
}
/// Area for drawing nodes, edges, select box, etc.
///
/// this raises [`BackgroundEvent`].
///
/// The following are the HTML attributes of this component.
/// The minimum style that does not interfere with operation is set.
/// ```text
/// class: "graph-area"
/// style: {
///     position: relative;
/// }
/// ```
#[styled_component(GraphArea)]
pub fn graph_area(
    GraphProps {
        children,
        onevent,
        node_ref,
    }: &GraphProps,
) -> Html {
    use_event_listeners::<Event, 3>(
        node_ref.clone(),
        [
            (
                "contextmenu",
                Box::new({
                    let onevent = onevent.clone();
                    move |e| {
                        e.prevent_default();
                        let e = e.dyn_ref::<MouseEvent>().unwrap();
                        onevent.emit(BackgroundEvent::ContextMenu(
                            get_mouse_pos_from_current_target(&e),
                        ))
                    }
                }),
            ),
            (
                "mousedown",
                Box::new({
                    let onevent = onevent.clone();
                    move |e| {
                        e.prevent_default();
                        let e = e.dyn_ref::<MouseEvent>().unwrap();
                        onevent.emit(BackgroundEvent::MouseDown {
                            button: e.button(),
                            pos: get_mouse_pos_from_current_target(&e),
                            is_shift_key_pressed: e.shift_key(),
                        })
                    }
                }),
            ),
            (
                "wheel",
                Box::new({
                    let onevent = onevent.clone();
                    move |e| {
                        e.prevent_default();
                        let e = e.dyn_ref::<WheelEvent>().unwrap();
                        let pos = get_mouse_pos_from_current_target(e);
                        let delta_y = e.delta_y();
                        onevent.emit(BackgroundEvent::Wheel { delta_y, pos })
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

/// Arguments of event callback in [`GraphArea`]
#[derive(Debug)]
pub enum BackgroundEvent {
    ContextMenu(Vec2),
    MouseDown {
        button: i16,
        pos: Vec2,
        is_shift_key_pressed: bool,
    },
    Wheel {
        pos: Vec2,
        delta_y: f64,
    },
}
