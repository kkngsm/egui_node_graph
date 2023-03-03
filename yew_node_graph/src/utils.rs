use glam::{vec2, Vec2};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{Element, MouseEvent};
use yew::{hook, use_effect_with_deps, NodeRef};

/// Get the center point of the element
pub fn get_center(r: &NodeRef) -> Option<Vec2> {
    r.cast::<web_sys::Element>().map(|e| {
        let rect = e.get_bounding_client_rect();
        let x = (rect.left() + rect.right()) as f32 * 0.5;
        let y = (rect.top() + rect.bottom()) as f32 * 0.5;
        vec2(x, y)
    })
}

/// Get the global upper-left position of the element
pub fn get_offset(r: &NodeRef) -> Option<Vec2> {
    r.cast::<web_sys::Element>().map(|e| {
        let rect = e.get_bounding_client_rect();
        let x = (rect.left()) as f32;
        let y = (rect.top()) as f32;
        vec2(x, y)
    })
}

/// Get relative mouse position of mouse event's current target
pub fn get_mouse_pos_from_current_target(e: &MouseEvent) -> Vec2 {
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
/// Add multiple event listeners
///```
/// use yew::{function_component, html, use_node_ref, Html};
/// use yew_node_graph::utils::use_event_listeners;
///
/// #[function_component]
/// fn EventListeners() -> Html {
///     let node_ref = use_node_ref();
///
///     use_event_listeners(
///         node_ref.clone(),
///         [
///             ("mousedown", Box::new(|_| log::debug!("mousedown"))),
///             ("click", Box::new(|_| log::debug!("click"))),
///         ],
///     );
///
///     html! { <div ref={node_ref}>{"EventListeners"}</div> }
/// }
///
/// ```
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
