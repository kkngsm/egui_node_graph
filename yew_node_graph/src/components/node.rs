use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    state::{
        Connections, InputId, InputParams, NodeId, OutputId, OutputParams, PortRefs,
        WidgetValueTrait,
    },
    utils::{get_offset_from_current_target, use_event_listeners},
    Vec2,
};
use stylist::yew::styled_component;
use yew::prelude::*;

use super::port::{
    unit::PortUnit,
    widget::{InputWidget, OutputWidget},
    Port, PortEvent,
};

#[derive(Properties)]
pub struct NodeProps<NodeData, DataType, ValueType, UserState> {
    pub data: Rc<crate::state::Node<NodeData>>,
    pub pos: Vec2,
    #[prop_or_default]
    pub is_selected: bool,
    pub onevent: Callback<NodeEvent>,

    pub input_params: InputParams<DataType, ValueType>,
    pub output_params: OutputParams<DataType>,
    pub connections: Connections,
    pub ports_ref: PortRefs,
    pub user_state: Rc<RefCell<UserState>>,
}
impl<NodeData, DataType, ValueType, UserState> PartialEq
    for NodeProps<NodeData, DataType, ValueType, UserState>
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.data, &other.data)
            && self.pos == other.pos
            && self.is_selected == other.is_selected
            && self.onevent == other.onevent
        // The following always return True, because RefCell is used.
        // && Rc::ptr_eq(&self.input_params, &other.input_params)
        // && Rc::ptr_eq(&self.output_params, &other.output_params)
        // && Rc::ptr_eq(&self.connections, &other.connections)
        // && Rc::ptr_eq(&self.user_state, &other.user_state)
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
        onevent,
        pos,
        is_selected,
        user_state,
        input_params,
        output_params,
        connections,
        ports_ref,
    }: &NodeProps<NodeData, DataType, ValueType, UserState>,
) -> Html
where
    NodeData: 'static,
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState> + 'static,
    UserState: 'static,
{
    let port_event = Callback::from({
        let onevent = onevent.clone();
        move |e| onevent.emit(NodeEvent::Port(e))
    });
    let input_ports = input_ports(
        &data.inputs,
        data.id,
        &data.user_data,
        port_event.clone(),
        input_params,
        connections,
        ports_ref,
        user_state.clone(),
    );
    let output_ports = output_ports(&data.outputs, port_event, output_params, ports_ref);

    let node = css! {r#"
position:absolute;
user-select:none;
"#};
    let node_ref = use_node_ref();
    use_event_listeners(
        node_ref.clone(),
        [
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
                        e.stop_propagation();
                        onevent.emit(NodeEvent::DragStart {
                            shift: get_offset_from_current_target(&e),
                            shift_key: e.shift_key(),
                        })
                    }
                }),
            ),
        ],
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
                {let onevent = onevent.clone();
                move  |e: MouseEvent|{
                    e.stop_propagation();
                    onevent.emit(NodeEvent::Delete)
                }}}
            >{"×"}</button>
            </div>
            {input_ports}
            {output_ports}
        </div>
    }
}

#[derive(Debug, Clone)]
pub enum NodeEvent {
    DragStart { shift: Vec2, shift_key: bool },
    Select { shift_key: bool },
    Delete,
    Port(PortEvent),
}

#[derive(Debug, Clone)]
pub enum NodeRendered {
    InputWidget(InputId, NodeRef),
    OutputWidget(OutputId, NodeRef),
    Node(NodeId, NodeRef),
}
#[allow(clippy::too_many_arguments)]
pub fn input_ports<NodeData, DataType, ValueType, UserState>(
    ports: &[(Rc<String>, InputId)],
    node_id: NodeId,
    node_data: &Rc<NodeData>,
    onevent: Callback<PortEvent>,
    input_params: &InputParams<DataType, ValueType>,
    connections: &Connections,
    ports_ref: &PortRefs,
    user_state: Rc<RefCell<UserState>>,
) -> Html
where
    NodeData: 'static,
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState> + 'static,
    UserState: 'static,
{
    let connections = connections.borrow();
    let widgets = ports.iter().map(|(name, id)| {
        let id = *id;
        let node_data = node_data.clone();
        let is_connected = connections.contains_key(id);
        let param = input_params.borrow()[id].clone();
        let user_state = user_state.clone();
        let node_ref = ports_ref.borrow()[id].clone();
        let onevent = onevent.clone();
        html! {
            <PortUnit>
                <Port<InputId, DataType>
                    {node_ref}
                    {id}
                    typ={param.typ.clone()}
                    is_should_draw={param.kind.is_should_draw()}
                    {onevent}
                />
                <InputWidget<NodeData, DataType, ValueType, UserState>
                    {name}
                    {is_connected}
                    {param}
                    {node_data}
                    {node_id}
                    {user_state}
                    key={id.to_string()}
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
pub fn output_ports<DataType>(
    ports: &[(Rc<String>, OutputId)],
    onevent: Callback<PortEvent>,
    params: &OutputParams<DataType>,
    ports_ref: &PortRefs,
) -> Html
where
    DataType: Display + PartialEq + Clone + 'static,
{
    let widgets = ports.iter().map(|(name, id)| {
        let id = *id;
        let name = name.clone();
        let param = params.borrow()[id].clone();
        let typ = param.typ.clone();
        let node_ref = ports_ref.borrow()[id].clone();
        let onevent = onevent.clone();
        html! {
        <PortUnit>
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
