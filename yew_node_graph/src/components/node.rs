use std::{fmt::Display, rc::Rc};

use crate::{
    state::{
        AnyParameterId, Graph, InputId, NodeDataTrait, NodeId, OutputId, PortRefs, WidgetValueTrait,
    },
    utils::{get_mouse_pos_from_current_target, use_event_listeners},
    Vec2,
};
use stylist::yew::styled_component;
use yew::prelude::*;

use super::port::{InputWidget, OutputWidget, Port, PortEvent, PortUnit};

/// Properties of [`Node`]
#[derive(Properties)]
pub struct NodeProps<NodeData, DataType, ValueType, UserState, UserResponse>
where
    UserState: PartialEq,
{
    pub data: Rc<crate::state::Node<NodeData>>,
    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,

    pub node_callback: Callback<(NodeId, NodeEvent)>,
    pub port_callback: Callback<(AnyParameterId, PortEvent)>,
    pub user_callback: Callback<UserResponse>,

    pub ports_ref: PortRefs,
    pub graph: Rc<Graph<NodeData, DataType, ValueType>>,
    pub user_state: UserState,
}
impl<NodeData, DataType, ValueType, UserState, UserResponse> PartialEq
    for NodeProps<NodeData, DataType, ValueType, UserState, UserResponse>
where
    UserState: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
            && self.pos == other.pos
            && self.is_selected == other.is_selected
            && self.node_callback == other.node_callback
            && self.port_callback == other.port_callback
            && self.user_callback == other.user_callback
            && self.user_state == other.user_state
            && Rc::ptr_eq(&self.graph, &other.graph)
        // The following always return True, because RefCell is used.
        // && Rc::ptr_eq(&self.ports_ref, &other.ports_ref)
    }
}

/// This have element title, delete button, input/output ports and User-defined ui inside.
///
/// title elements has `node-title` class,
/// delete-button elements has `node-delete` class,
/// ports have "ports"
/// User-defined ui has a `bottom-ui` class.
///
///
/// The following are the HTML attributes of this component.
/// The minimum style that does not interfere with operation is set.
/// ```text
/// class: "node"
/// data-is-selected: `props.is_selected.to_string()`
/// style: {
///     position:absolute;
///     user-select:none;
///     left: {}px;
///     top: {}px;
/// }
/// ```

#[styled_component(Node)]
pub fn node<NodeData, DataType, ValueType, UserState, UserResponse>(
    NodeProps {
        data,
        pos,
        is_selected,
        user_state,
        node_callback,
        port_callback,
        user_callback,
        graph,
        ports_ref,
    }: &NodeProps<NodeData, DataType, ValueType, UserState, UserResponse>,
) -> Html
where
    NodeData: NodeDataTrait<
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
            Response = UserResponse,
        > + 'static,
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState, Response = UserResponse>
        + 'static,
    UserState: Clone + PartialEq + 'static,
    UserResponse: 'static,
{
    let id = data.id;
    let port_event = Callback::from({
        let port_callback = port_callback.clone();
        move |p| port_callback.emit(p)
    });
    let input_ports = input_ports(
        &data.inputs,
        data.id,
        &data.user_data,
        port_event.clone(),
        graph,
        ports_ref,
        user_state,
        user_callback,
    );
    let output_ports = output_ports(&data.outputs, port_event, graph, ports_ref);

    let node = css! {r#"
position:absolute;
user-select:none;
"#};
    let node_ref = use_node_ref();
    use_event_listeners(
        node_ref.clone(),
        [(
            "mousedown",
            Box::new({
                let node_callback = node_callback.clone();
                move |e| {
                    e.stop_propagation();
                    node_callback.emit((
                        id,
                        NodeEvent::MouseDown {
                            shift: get_mouse_pos_from_current_target(&e),
                            shift_key: e.shift_key(),
                        },
                    ))
                }
            }),
        )],
    );
    let bottom_ui = data.user_data.bottom_ui(
        data.id,
        graph,
        user_state,
        Callback::from({
            let user_callback = user_callback.clone();
            move |user_response| user_callback.emit(user_response)
        }),
    );
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
            <div class={"node-title"}>{&data.label}
            <button class={"node-delete"}
                onclick={
                {let node_callback = node_callback.clone();
                move |e: MouseEvent|{
                    e.stop_propagation();
                    node_callback.emit((id, NodeEvent::Delete))
                }}}
            >{"x"}</button>
            </div>
            {input_ports}
            {output_ports}
            <div class={"bottom_ui"}>{bottom_ui}</div>
        </div>
    }
}

/// Arguments of event callback in [`Node`]
#[derive(Debug, Clone)]
pub enum NodeEvent {
    MouseDown { shift: Vec2, shift_key: bool },
    Delete,
}

#[allow(clippy::too_many_arguments)]
fn input_ports<NodeData, DataType, ValueType, UserState, UserResponse>(
    ports: &[(Rc<String>, InputId)],
    node_id: NodeId,
    node_data: &Rc<NodeData>,
    onevent: Callback<(AnyParameterId, PortEvent)>,
    graph: &Graph<NodeData, DataType, ValueType>,
    ports_ref: &PortRefs,
    user_state: &UserState,
    user_callback: &Callback<UserResponse>,
) -> Html
where
    NodeData: 'static,
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState, Response = UserResponse>
        + 'static,
    UserState: PartialEq + Clone + 'static,
    UserResponse: 'static,
{
    let widgets = ports.iter().map(|(name, id)| {
        let id = *id;
        let node_data = node_data.clone();
        let is_connected = graph.connections.contains_key(id);
        let param = graph.inputs[id].clone();
        let node_ref = ports_ref.borrow()[id].clone();
        let onevent = onevent.clone();
        html! {
            <PortUnit key={id.to_string()}>
                <Port<InputId, DataType>
                    {node_ref}
                    {id}
                    typ={param.typ.clone()}
                    is_should_draw={param.kind.is_should_draw()}
                    {onevent}
                />
                <InputWidget<NodeData, DataType, ValueType, UserState, UserResponse>
                    {name}
                    {is_connected}
                    {param}
                    {node_data}
                    {node_id}
                    user_state={user_state.clone()}
                    user_callback={user_callback.clone()}
                />
            </PortUnit>
        }
    });
    html! {
        <div class={"ports"} data-io={"input"}>
            {for widgets}
        </div>
    }
}
fn output_ports<NodeData, DataType, ValueType>(
    ports: &[(Rc<String>, OutputId)],
    onevent: Callback<(AnyParameterId, PortEvent)>,
    graph: &Graph<NodeData, DataType, ValueType>,
    ports_ref: &PortRefs,
) -> Html
where
    DataType: Display + PartialEq + Clone + 'static,
{
    let widgets = ports.iter().map(|(name, id)| {
        let id = *id;
        let name = name.clone();
        let param = graph.outputs[id].clone();
        let typ = param.typ.clone();
        let node_ref = ports_ref.borrow()[id].clone();
        let onevent = onevent.clone();
        html! {
        <PortUnit key={id.to_string()}>
            <OutputWidget<DataType>
                {name}
                {param}
            />
            <Port<OutputId, DataType>
                {node_ref}
                {id}
                {typ}
                is_should_draw=true
                {onevent}
            />
        </PortUnit>
        }
    });
    html! {
        <div class={"ports"} data-io={"output"}>
            {for widgets}
        </div>
    }
}
