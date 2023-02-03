use crate::Vec2;
use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NodeProps {
    pub title: String,
    pub onevent: Option<Callback<NodeEvent>>,
    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,
}
#[styled_component(Node)]
pub fn node(
    NodeProps {
        title,
        onevent,
        pos,
        is_selected,
    }: &NodeProps,
) -> Html {
    let onevent = onevent.clone();
    let node = css! {r#"
position:absolute;
user-select:none;
display:inline-block;
"#};
    html! {
        <div
            class={classes![
                node,
                "node"
            ]}
            style={format!("left:{}px;top:{}px;", pos.x, pos.y)}
            onmousedown={move |e:MouseEvent| if let Some(c) = onevent.as_ref(){
                e.stop_propagation();
                c.emit(NodeEvent::MouseDown);
            }}
            onclick={|e:MouseEvent| e.stop_propagation()}
            data-is-selected={is_selected.to_string()}
        >
            <div>{title}</div>
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeEvent {
    MouseDown,
}
