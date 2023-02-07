use std::rc::Rc;

use crate::{
    state::{InputId, InputParams, OutputId, OutputParams},
    utils::{get_offset_from_current_target, on_event, use_event_listeners},
    Vec2,
};
use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties)]
pub struct NodeProps<NodeData, DataType, ValueType> {
    pub data: Rc<crate::state::Node<NodeData>>,
    pub input_params: InputParams<DataType, ValueType>,
    pub output_params: OutputParams<DataType>,

    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,
    pub onevent: Option<Callback<NodeEvent>>,
}
impl<NodeData, DataType, ValueType> PartialEq for NodeProps<NodeData, DataType, ValueType> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
            && Rc::ptr_eq(&self.input_params, &other.input_params)
            && Rc::ptr_eq(&self.output_params, &other.output_params)
            && self.pos == other.pos
            && self.is_selected == other.is_selected
            && self.onevent == other.onevent
    }
}

/// Node component
/// if this node is selected, its html attribute `data-is-selected` is `true`
/// this components have `node` class
///
/// # Default style
/// ```css
/// position:absolute;
/// user-select:none;
/// ```
#[styled_component(Node)]
pub fn node<NodeData, DataType: ToString, ValueType>(
    NodeProps {
        data,
        input_params,
        output_params,
        onevent,
        pos,
        is_selected,
    }: &NodeProps<NodeData, DataType, ValueType>,
) -> Html {
    let input_ports = input_ports(&data.inputs, &input_params);
    let output_ports = output_ports(&data.outputs, &output_params);
    let node = css! {r#"
position:absolute;
user-select:none;
"#};

    let node_ref = use_event_listeners([
        (
            "click",
            Box::new(on_event(onevent.clone(), |e| {
                e.stop_propagation();
                NodeEvent::Select {
                    shift_key: e.shift_key(),
                }
            })),
        ),
        (
            "mousedown",
            Box::new(on_event(onevent.clone(), |e| NodeEvent::DragStart {
                gap: get_offset_from_current_target(&e),
                shift_key: e.shift_key(),
            })),
        ),
    ]);
    html! {
        <div
            ref={node_ref}
            class={classes![
                node,
                "node"
            ]}
            style={format!("left:{}px;top:{}px;", pos.x, pos.y)}
            data-is-selected={is_selected.to_string()}
        >
            <div class={"node-title"}>{&data.label}</div>
            {input_ports}
            {output_ports}
        </div>
    }
}

#[derive(Debug, Clone)]
pub enum NodeEvent {
    DragStart { gap: Vec2, shift_key: bool },
    Select { shift_key: bool },
}

pub fn input_ports<DataType: ToString, ValueType>(
    ports: &[(String, InputId)],
    graph: &InputParams<DataType, ValueType>,
) -> Html {
    let ports = ports.iter().map(|(label, id)| {
        let typ = &graph[*id].typ;
        html! {
            <div class={"port-wrap"}>
                <div class={"port"} data-type={typ.to_string()}/>
                <div class={"port-label"}>{label}</div>
            </div>
        }
    });
    html! {
        <div class={"ports"} data-io={"input"}>
            {for ports}
        </div>
    }
}
pub fn output_ports<DataType: ToString>(
    ports: &[(String, OutputId)],
    graph: &OutputParams<DataType>,
) -> Html {
    let ports = ports.iter().map(|(label, id)| {
        let typ = &graph[*id].typ;
        html! {
            <div class={"port-wrap"}>
                <div class={"port-label"}>{label}</div>
                <span class={"port"} data-type={typ.to_string()}/>
            </div>
        }
    });
    html! {
        <div class={"ports"} data-io={"output"}>
            {for ports}
        </div>
    }
}
