use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub children: Children,
    pub onevent: Option<Callback<BackgroundEvent>>,
}
#[styled_component(GraphArea)]
pub fn graph_area(GraphProps { children, onevent }: &GraphProps) -> Html {
    let graph_area = css!(
        r#"
position: relative;
overflow:hidden;
"#
    );
    html! {
        <div
            class={classes![graph_area,"graph-area"]}
            oncontextmenu={stop_propagation(onevent.clone(), |e| BackgroundEvent::ContextMenu(e))}
            onclick={stop_propagation(onevent.clone(), |e| BackgroundEvent::Click(e))}
        >
            { for children.iter() }
        </div>
    }
}

fn stop_propagation<WrappedEvent>(
    callback: Option<Callback<WrappedEvent>>,
    wrap: impl Fn(MouseEvent) -> WrappedEvent,
) -> impl Fn(MouseEvent) {
    move |e: MouseEvent| {
        if let Some(c) = callback.as_ref() {
            e.stop_propagation();
            e.prevent_default();
            c.emit(wrap(e))
        }
    }
}

#[derive(Debug)]
pub enum BackgroundEvent {
    ContextMenu(MouseEvent),
    Click(MouseEvent),
}
