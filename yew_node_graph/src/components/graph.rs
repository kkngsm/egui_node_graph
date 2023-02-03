use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    pub children: Children,
    pub onevent: Option<Callback<BackgroundEvent>>,
}
#[styled_component(GraphArea)]
pub fn graph_area(GraphProps { children, onevent }: &GraphProps) -> Html {
    let oncontextmenu = onevent.clone();
    let onclick = onevent.clone();
    let graph_area = css!(
        r#"
position: relative;
overflow:hidden;
"#
    );
    html! {
        <div
            class={classes![graph_area,"graph-area"]}
            oncontextmenu={move |e:MouseEvent| if let Some(c) = oncontextmenu.as_ref() {
                e.prevent_default();
                c.emit(BackgroundEvent::ContextMenu(e))
            }}
            onclick={move |e:MouseEvent| if let Some(c) = onclick.as_ref() {
                c.emit(BackgroundEvent::Click(e))
            }}
        >
            { for children.iter() }
        </div>
    }
}

#[derive(Debug)]
pub enum BackgroundEvent {
    ContextMenu(MouseEvent),
    Click(MouseEvent),
}
