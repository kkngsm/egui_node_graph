use glam::{vec2, Vec2};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsValue,
};
use web_sys::HtmlElement;
use yew::{hook, use_effect_with_deps};

#[wasm_bindgen(module = "/src/hooks/use_drag.js")]
extern "C" {
    fn drag(
        element: HtmlElement,
        on_mouse_down: JsValue,
        on_mouse_move: JsValue,
        on_mouse_up: JsValue,
    );
}

#[hook]
pub fn use_drag(on_drag: impl FnMut(DragEvent) + 'static) -> yew::NodeRef {
    let div_ref = yew::use_node_ref();

    {
        let div_ref = div_ref.clone();
        let on_drag = Rc::new(RefCell::new(on_drag));
        use_effect_with_deps(
            move |div_ref| {
                let div = div_ref
                    .cast::<HtmlElement>()
                    .expect("div_ref not attached to div element");

                drag(
                    div,
                    on_mouse_down(on_drag.clone()),
                    on_move(on_drag.clone()),
                    on_mouse_up(on_drag.clone()),
                );
            },
            div_ref,
        );
    }
    div_ref
}

#[derive(Debug, Clone, Copy)]

pub enum DragEvent {
    Start,
    Move(Vec2, Vec2),
    End,
}

fn on_mouse_down(on_drag: Rc<RefCell<impl FnMut(DragEvent) + 'static>>) -> JsValue {
    Closure::<dyn FnMut()>::new(move || on_drag.borrow_mut()(DragEvent::Start)).into_js_value()
}

fn on_move(on_drag: Rc<RefCell<impl FnMut(DragEvent) + 'static>>) -> JsValue {
    Closure::<dyn FnMut(f32, f32, f32, f32)>::new(move |x, y, dx, dy| {
        on_drag.borrow_mut()(DragEvent::Move(vec2(x, y), vec2(dx, dy)))
    })
    .into_js_value()
}

fn on_mouse_up(on_drag: Rc<RefCell<impl FnMut(DragEvent) + 'static>>) -> JsValue {
    Closure::<dyn FnMut()>::new(move || on_drag.borrow_mut()(DragEvent::End)).into_js_value()
}
