use glam::{vec2, Vec2};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Element, MouseEvent};
use yew::{hook, use_effect_with_deps, NodeRef};

pub fn get_center(r: &NodeRef) -> Vec2 {
    let element = r.cast::<web_sys::Element>().unwrap();
    let rect = element.get_bounding_client_rect();
    let x = (rect.left() + rect.right()) as f32 * 0.5;
    let y = (rect.top() + rect.bottom()) as f32 * 0.5;
    vec2(x, y)
}

pub fn get_near<Id>(mouse_pos: Vec2, th: f32) -> impl Fn((Id, &NodeRef)) -> Option<(Id, Vec2)> {
    move |(id, n)| {
        let pos = get_center(n);
        (pos.distance_squared(mouse_pos) < th).then_some((id, pos))
    }
}
pub fn get_offset_from_target(e: &MouseEvent) -> Vec2 {
    if let Some(target) = e
        .target()
        .and_then(|event_target: web_sys::EventTarget| event_target.dyn_into::<Element>().ok())
    {
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
        .and_then(|event_target: web_sys::EventTarget| event_target.dyn_into::<Element>().ok())
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
#[allow(clippy::type_complexity)]
pub fn use_event_listeners<const N: usize>(
    node_ref: NodeRef,
    events_callbacks: [(&'static str, Box<dyn Fn(MouseEvent)>); N],
) {
    use_effect_with_deps(
        move |div_ref| {
            let div = div_ref
                .cast::<Element>()
                .expect("div_ref not attached to div element");
            let events_callbacks = events_callbacks
                .map(|(event, callback)| (event, Closure::<dyn Fn(MouseEvent)>::wrap(callback)));
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
        node_ref,
    );
}
