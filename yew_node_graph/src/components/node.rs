use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    state::{InputId, InputParams, NodeId, OutputId, OutputParams, WidgetValueTrait},
    utils::{get_offset_from_current_target, use_event_listeners},
    Vec2,
};
use stylist::yew::styled_component;
use yew::prelude::*;

use super::{port::PortEvent, Port};
#[derive(Properties)]
pub struct NodeProps<NodeData, DataType, ValueType, UserState> {
    pub data: Rc<crate::state::Node<NodeData>>,
    pub input_params: InputParams<DataType, ValueType>,
    pub output_params: OutputParams<DataType>,

    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,
    pub onevent: Callback<NodeEvent>,
    pub user_state: Rc<RefCell<UserState>>,
}
impl<NodeData, DataType, ValueType, UserState> PartialEq
    for NodeProps<NodeData, DataType, ValueType, UserState>
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
            && Rc::ptr_eq(&self.input_params, &other.input_params)
            && Rc::ptr_eq(&self.output_params, &other.output_params)
            && self.pos == other.pos
            && self.is_selected == other.is_selected
            && self.onevent == other.onevent
            && Rc::ptr_eq(&self.user_state, &other.user_state)
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
pub fn node<NodeData, DataType, ValueType, UserState>(
    NodeProps {
        data,
        input_params,
        output_params,
        onevent,
        pos,
        is_selected,
        user_state,
    }: &NodeProps<NodeData, DataType, ValueType, UserState>,
) -> Html
where
    DataType: Display + Clone + PartialEq + 'static,
    ValueType: WidgetValueTrait<UserState = UserState, NodeData = NodeData> + Clone,
{
    let input_ports = input_ports(
        &data.inputs,
        &input_params,
        data.id,
        &data.user_data,
        &mut *user_state.borrow_mut(),
        onevent.clone(),
    );
    let output_ports = output_ports(&data.outputs, &output_params, onevent.clone());
    let node = css! {r#"
position:absolute;
user-select:none;
"#};

    let node_ref = use_event_listeners([
        (
            "click",
            Box::new({
                let onevent = onevent.clone();
                move |e| {
                    e.stop_propagation();
                    onevent.emit(NodeEvent::Select {
                        shift_key: e.shift_key(),
                    })
                }
            }),
        ),
        (
            "mousedown",
            Box::new({
                let onevent = onevent.clone();
                move |e| {
                    onevent.emit(NodeEvent::DragStart {
                        gap: get_offset_from_current_target(&e),
                        shift_key: e.shift_key(),
                    })
                }
            }),
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
    Port(PortEvent),
}

pub fn input_ports<DataType, ValueType, NodeData, UserState>(
    ports: &[(String, InputId)],
    input_params: &InputParams<DataType, ValueType>,
    node_id: NodeId,
    node_data: &NodeData,
    user_state: &mut UserState,
    onevent: Callback<NodeEvent>,
) -> Html
where
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState> + Clone,
{
    let ports = ports.iter().map(|(param_name, id)| {
        let id = *id;
        let input_params = &input_params.borrow()[id];
        let widget = input_params
            .value
            .value_widget(param_name, node_id, user_state, node_data);
        let onevent = onevent.clone();
        html! {
            <div class={"port-wrap"}>
                <Port<InputId, DataType> {id}
                typ={input_params.typ.clone()}
                is_should_draw={input_params.kind.is_should_draw()}
                onevent={move |event| {
                    onevent.emit(NodeEvent::Port(event))
                }}/>
                <div class={"port-widget"}>
                    {widget}
                </div>
            </div>
        }
    });
    html! {
        <div class={"ports"} data-io={"input"}>
            {for ports}
        </div>
    }
}
pub fn output_ports<DataType>(
    ports: &[(String, OutputId)],
    output_params: &OutputParams<DataType>,
    onevent: Callback<NodeEvent>,
) -> Html
where
    DataType: Display + PartialEq + Clone + 'static,
{
    let ports = ports.iter().map(|(label, id)| {
        let id = *id;
        let typ = output_params.borrow()[id].typ.clone();
        let onevent = onevent.clone();
        html! {
            <div class={"port-wrap"}>
                <div class={"port-widget"}>{label}</div>
                <Port<OutputId, DataType> {id} {typ} is_should_draw={true} onevent={move |event| {
                    onevent.emit(NodeEvent::Port(event))
                }}
                    />
            </div>
        }
    });
    html! {
        <div class={"ports"} data-io={"output"}>
            {for ports}
        </div>
    }
}
