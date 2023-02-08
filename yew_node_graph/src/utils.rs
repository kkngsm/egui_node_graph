use glam::{vec2, Vec2};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::MouseEvent;
use yew::{hook, use_effect_with_deps, use_node_ref, NodeRef};

pub fn get_offset_from_target(e: &MouseEvent) -> Vec2 {
    if let Some(target) = e.target().and_then(|event_target: web_sys::EventTarget| {
        event_target.dyn_into::<web_sys::Element>().ok()
    }) {
        let rect: web_sys::DomRect = target.get_bounding_client_rect();
        let x = e.client_x() as f32 - rect.left() as f32;
        let y = e.client_y() as f32 - rect.top() as f32;
        vec2(x, y)
    } else {
        Vec2::ZERO
    }
}

pub fn get_offset_from_current_target(e: &MouseEvent) -> Vec2 {
    if let Some(target) = e
        .current_target()
        .and_then(|event_target: web_sys::EventTarget| {
            event_target.dyn_into::<web_sys::Element>().ok()
        })
    {
        let rect: web_sys::DomRect = target.get_bounding_client_rect();
        let x = e.client_x() as f32 - rect.left() as f32;
        let y = e.client_y() as f32 - rect.top() as f32;
        vec2(x, y)
    } else {
        Vec2::ZERO
    }
}

#[hook]
pub fn use_event_listeners<const N: usize>(
    events_callbacks: [(&'static str, Box<dyn Fn(MouseEvent)>); N],
) -> NodeRef {
    let div_ref = use_node_ref();
    {
        let div_ref = div_ref.clone();

        use_effect_with_deps(
            move |div_ref| {
                let div = div_ref
                    .cast::<web_sys::Element>()
                    .expect("div_ref not attached to div element");
                let events_callbacks = events_callbacks.map(|(event, callback)| {
                    (event, Closure::<dyn Fn(MouseEvent)>::wrap(callback))
                });
                for (event, callback) in &events_callbacks {
                    div.add_event_listener_with_callback(event, callback.as_ref().unchecked_ref())
                        .unwrap();
                }
                move || {
                    for (event, callback) in &events_callbacks {
                        div.remove_event_listener_with_callback(
                            event,
                            callback.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    }
                }
            },
            div_ref,
        );
    }
    div_ref
}
