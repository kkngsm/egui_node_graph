use std::fmt::{Debug, Display};

use crate::components::contextmenu::ContextMenu;
use crate::components::edge::Edge;
use crate::components::graph::{BackgroundEvent, GraphArea};
use crate::components::node::{Node, NodeEvent};
use crate::components::port::PortEvent;
use crate::components::select_box::SelectBox;
use crate::state::basic::BasicGraphEditorState;
use crate::state::{
    AnyParameterId, ConnectTo, ConnectionInProgress, DragState, NodeDataTrait, NodeId,
    NodeTemplateIter, NodeTemplateTrait, UserResponseTrait, WidgetValueTrait,
};
use crate::utils::{get_center, get_offset, get_offset_from_current_target};
use crate::Vec2;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

pub struct BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate> {
    state: BasicGraphEditorState<NodeData, DataType, ValueType, NodeTemplate>,
}
#[derive(Debug, Clone)]
pub enum GraphMessage<NodeTemplate, UserResponse> {
    SelectNode {
        id: NodeId,
        shift_key: bool,
    },
    DeleteNode {
        id: NodeId,
    },

    DragStartPort(AnyParameterId),
    DragStartNode {
        id: NodeId,
        shift: Vec2,
        shift_key: bool,
    },
    DragStartBackground {
        pos: Vec2,
        is_shift_key_pressed: bool,
    },

    Dragging(Vec2),
    EnterPort(AnyParameterId),
    LeavePort(AnyParameterId),
    DragEnd,

    // NodeFinder Event
    OpenNodeFinder(Vec2),
    CreateNode(NodeTemplate),

    User(UserResponse),
    None,
}

/// Props for [`BasicGraphEditor`]
#[derive(Properties, PartialEq)]
pub struct BasicGraphEditorProps<UserState: PartialEq, UserResponse: PartialEq> {
    pub user_state: UserState,
    pub callback: Callback<UserResponse>,
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse> Component
    for BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate>
where
    NodeData: 'static
        + NodeDataTrait<
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
            Response = UserResponse,
        >,
    UserState: 'static + Clone + PartialEq,
    NodeTemplate: 'static
        + NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        >
        + NodeTemplateIter<Item = NodeTemplate>
        + PartialEq
        + Copy
        + Debug,
    DataType: 'static + Display + PartialEq + Clone,
    ValueType: 'static
        + WidgetValueTrait<UserState = UserState, NodeData = NodeData, Response = UserResponse>
        + Clone,
    UserResponse: 'static + UserResponseTrait,
{
    type Message = GraphMessage<NodeTemplate, UserResponse>;
    type Properties = BasicGraphEditorProps<UserState, UserResponse>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: Default::default(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        // log::debug!("{:?}", &msg);
        let BasicGraphEditorProps {
            user_state,
            callback,
        } = ctx.props();
        let state = &mut self.state;
        match msg {
            GraphMessage::SelectNode { id, shift_key } => {
                if !state.selected_nodes.contains(&id) {
                    if !shift_key {
                        state.selected_nodes.clear();
                    }
                    state.selected_nodes.insert(id);
                }
                true
            }
            GraphMessage::DeleteNode { id } => {
                let (_node, _disc_events) = state.graph.borrow_mut().remove_node(id);
                true
            }
            GraphMessage::DragStartNode {
                id,
                shift,
                shift_key,
            } => {
                if state.selected_nodes.contains(&id) {
                    state.set_drag_event::<Self::Message>(
                        ctx.link().callback(|_| GraphMessage::DragEnd),
                        ctx.link().callback(|e: web_sys::Event| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            GraphMessage::Dragging(get_offset_from_current_target(e))
                        }),
                    );
                    state.drag_event = Some(DragState::MoveNode {
                        id,
                        shift,
                        is_moved: false,
                        is_shift_key_pressed: shift_key,
                    });
                }
                false
            }
            GraphMessage::DragStartPort(id) => {
                state.set_drag_event::<Self::Message>(
                    ctx.link().callback(|_| GraphMessage::DragEnd),
                    ctx.link().callback(|e: web_sys::Event| {
                        let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                        GraphMessage::Dragging(get_offset_from_current_target(e))
                    }),
                );
                let pos = state
                    .port_refs
                    .borrow()
                    .get(id)
                    .and_then(get_center)
                    .unwrap_or_default();

                if let AnyParameterId::Input(input) = id {
                    if let Some(output) = state.graph.borrow_mut().connections.remove(input) {
                        state.drag_event = Some(DragState::ConnectPort((output, pos).into()));
                    } else {
                        state.drag_event = Some(DragState::ConnectPort((input, pos).into()));
                    }
                } else {
                    state.drag_event = Some(DragState::ConnectPort((id, pos).into()));
                }

                false
            }
            GraphMessage::DragStartBackground {
                pos,
                is_shift_key_pressed,
            } => {
                state.set_drag_event::<Self::Message>(
                    ctx.link().callback(|_| GraphMessage::DragEnd),
                    ctx.link().callback(|e: web_sys::Event| {
                        let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                        GraphMessage::Dragging(get_offset_from_current_target(e))
                    }),
                );
                if !is_shift_key_pressed {
                    state.selected_nodes.clear();
                }
                state.drag_event = Some(DragState::SelectBox {
                    start: pos,
                    end: pos,
                });
                true
            }
            GraphMessage::Dragging(mouse_pos) => {
                match state.drag_event.as_mut() {
                    Some(DragState::SelectBox { end, .. }) => {
                        *end = mouse_pos;
                    }
                    Some(DragState::MoveNode {
                        id,
                        shift,
                        is_moved,
                        ..
                    }) => {
                        let pos = mouse_pos - *shift;
                        let selected_pos = state.node_positions[*id];
                        let drag_delta = pos - selected_pos;
                        for id in &state.selected_nodes {
                            let id = *id;
                            state.node_positions[id] += drag_delta;
                        }
                        *is_moved = true;
                    }
                    Some(DragState::ConnectPort(c)) => {
                        if let ConnectionInProgress::FromInput {
                            dest: ConnectTo::Pos(pos),
                            ..
                        }
                        | ConnectionInProgress::FromOutput {
                            dest: ConnectTo::Pos(pos),
                            ..
                        } = c
                        {
                            *pos = mouse_pos;
                        }
                    }
                    None => (),
                }
                true
            }
            GraphMessage::EnterPort(id) => {
                if let Some(DragState::ConnectPort(c)) = state.drag_event.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| state.graph.borrow().param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        c.to_id(id);
                    }
                }

                true
            }
            GraphMessage::LeavePort(id) => {
                if let Some(DragState::ConnectPort(c)) = state.drag_event.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| state.graph.borrow().param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        let offset = get_offset(&state.graph_ref).unwrap_or_default();
                        let port_pos = state
                            .port_refs
                            .borrow()
                            .get(id)
                            .and_then(get_center)
                            .map(|p| p - offset)
                            .unwrap_or_default();
                        c.to_pos(port_pos);
                    }
                }
                false
            }
            GraphMessage::DragEnd => {
                state._drag_event_listener = None;
                state.node_finder.is_showing = false;
                let mut graph = state.graph.borrow_mut();
                match state.drag_event.take() {
                    Some(DragState::SelectBox { start, end }) => {
                        let min = start.min(end);
                        let max = start.max(end);
                        for id in state.node_positions.iter().flat_map(|(id, pos)| {
                            (min.cmplt(*pos).all() && pos.cmplt(max).all()).then_some(id)
                        }) {
                            state.selected_nodes.insert(id);
                        }
                    }
                    Some(DragState::ConnectPort(c)) => {
                        // Connect to Port
                        if let Some((&output, &input)) = c.pair() {
                            if graph.param_typ_eq(output, input) {
                                graph.connections.insert(input, output);
                            }
                        }
                    }
                    Some(DragState::MoveNode {
                        id,
                        is_moved,
                        is_shift_key_pressed,
                        ..
                    }) => {
                        if !is_moved {
                            if is_shift_key_pressed {
                                if !state.selected_nodes.remove(&id) {
                                    state.selected_nodes.insert(id);
                                }
                            } else {
                                state.selected_nodes.clear();
                                state.selected_nodes.insert(id);
                            }
                        }
                    }
                    _ => (),
                }

                true
            }
            GraphMessage::CreateNode(template) => {
                let new_node = state.graph.borrow_mut().add_node(
                    template.node_graph_label(user_state),
                    template.user_data(user_state),
                    |graph, node_id| template.build_node(graph, user_state, node_id),
                );
                state.node_positions.insert(new_node, state.node_finder.pos);
                state.selected_nodes.insert(new_node);

                let node = &state.graph.borrow()[new_node];
                for input in node.input_ids() {
                    state
                        .port_refs
                        .borrow_mut()
                        .input
                        .insert(input, Default::default());
                }
                for output in node.output_ids() {
                    state
                        .port_refs
                        .borrow_mut()
                        .output
                        .insert(output, Default::default());
                }
                true
            }
            GraphMessage::OpenNodeFinder(pos) => {
                state.node_finder.is_showing = true;
                state.node_finder.pos = pos;
                true
            }
            GraphMessage::User(res) => {
                let rerender = res.should_rerender();
                callback.emit(res);
                rerender
            }
            GraphMessage::None => false,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        use GraphMessage::*;
        let BasicGraphEditorProps { user_state, .. } = ctx.props();
        let state = &self.state;
        let graph = state.graph.borrow();
        let nodes = graph.nodes.keys().map(|id| {
            let user_state = user_state.to_owned();
            let node_event = ctx.link().callback(move |e| match e {
                NodeEvent::Select { shift_key } => SelectNode { id, shift_key },
                NodeEvent::Delete => DeleteNode { id },
                NodeEvent::DragStart { shift, shift_key } => DragStartNode {
                    id,
                    shift,
                    shift_key,
                },
                NodeEvent::Port(PortEvent::MouseDown(id)) => DragStartPort(id),
                NodeEvent::Port(PortEvent::MouseEnter(id)) => EnterPort(id),
                NodeEvent::Port(PortEvent::MouseLeave(id)) => LeavePort(id),
                NodeEvent::User(res) => User(res),
            });
            html! {<Node<NodeData, DataType, ValueType, UserState, UserResponse>
                key={id.to_string()}
                data={graph[id].clone()}
                pos={state.node_positions[id]}
                is_selected={state.selected_nodes.contains(&id)}
                onevent={node_event}
                user_state={user_state}
                graph={state.graph.clone()}
                ports_ref={state.port_refs.clone()}
            />}
        });

        let background_event = ctx.link().callback(|e: BackgroundEvent| match e {
            BackgroundEvent::ContextMenu(pos) => OpenNodeFinder(pos),
            BackgroundEvent::MouseDown {
                button,
                pos,
                is_shift_key_pressed,
            } if button != 2 => DragStartBackground {
                pos,
                is_shift_key_pressed,
            },
            _ => None,
        });

        let edges_and_drag =get_offset(&state.graph_ref).map(|offset|{
            let port_refs = state.port_refs.borrow();
            let drag = match &state.drag_event {
                Some(DragState::ConnectPort(c)) => {
                    let connection_in_progress = match c {
                        ConnectionInProgress::FromInput {
                            src: from,
                            dest: to,
                        } => Some((
                            to.map_pos(|id| {
                                port_refs
                                    .output
                                    .get(*id)                            .and_then(get_center)
                                    .map(|p| p - offset)
                                    .unwrap_or_default()
                            }),
                            port_refs
                                .input
                                .get(*from)                            .and_then(get_center)
                                .map(|p| p - offset)
                                .unwrap_or_default(),
                            graph.inputs[*from].typ.clone(),
                        )),
                        ConnectionInProgress::FromOutput {
                            src: from,
                            dest: to,
                        } => Some((
                            port_refs
                                .output
                                .get(*from)
                                .and_then(get_center)
                                .map(|p| p - offset)
                                .unwrap_or_default(),
                            to.map_pos(|id| {
                                port_refs
                                    .input
                                    .get(*id)
                                    .and_then(get_center)
                                    .map(|p| p - offset)
                                    .unwrap_or_default()
                            }),
                            graph.outputs[*from].typ.clone(),
                        )),
                    };
                    connection_in_progress.map(|(output, input, typ)| {
                        html! {
                            <Edge<DataType> {output} {input} {typ}/>
                        }
                    })
                }
                Some(DragState::SelectBox { start, end }) => {
                    let start = *start;
                    let end = *end;
                    Some(html! {
                        <SelectBox {start} {end} />
                    })
                }
                _ => Option::None,
            };
            let graph = state.graph.borrow();
            let edges = graph.connections.iter().map(|(input, output)| {
                let typ = state.graph.borrow().inputs[input].typ.clone();
                let output_pos =state
                    .port_refs.borrow()
                    .output
                    .get(*output)
                    .and_then(get_center)
                    .map(|p| p - offset)
                    .unwrap_or_default();
                let input_pos =state
                    .port_refs.borrow()
                    .input
                    .get(input)
                    .and_then(get_center)
                    .map(|p| p - offset)
                    .unwrap_or_default();
                html! {<Edge<DataType> key={input.to_string()} output={output_pos} input={input_pos} {typ} />}
            });

            html! {
                <>
                {for edges}
                {drag}
                </>
            }
        });

        html! {
            <GraphArea
                node_ref={state.graph_ref.clone()}
                onevent={background_event}
            >
            {for nodes}
            {edges_and_drag}
            <BasicNodeFinder<NodeTemplate, UserState>
                is_showing={state.node_finder.is_showing}
                pos={state.node_finder.pos}
                user_state={user_state.to_owned()}
                onevent={ctx.link().callback(|t| CreateNode(t))}
            />
            </GraphArea>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct BasicNodeFinderProps<NodeTemplate, UserState>
where
    NodeTemplate: PartialEq,
    UserState: PartialEq,
{
    pub is_showing: bool,
    pub pos: Vec2,
    pub user_state: UserState,
    pub onevent: Callback<NodeTemplate>,
}

#[function_component(BasicNodeFinder)]
pub fn basic_finder<NodeTemplate, UserState>(
    BasicNodeFinderProps {
        is_showing,
        pos,
        user_state,
        onevent,
    }: &BasicNodeFinderProps<NodeTemplate, UserState>,
) -> Html
where
    NodeTemplate: NodeTemplateTrait<UserState = UserState>
        + NodeTemplateIter<Item = NodeTemplate>
        + PartialEq
        + Copy
        + 'static,
    UserState: PartialEq,
{
    let buttons = NodeTemplate::all_kinds().into_iter().map(|t| {
        let onevent = onevent.clone();
        html! {
            <li><button onclick={move |_| onevent.emit(t)}>{t.node_finder_label(user_state)}</button></li>
        }
    });
    html! {
        <ContextMenu pos={*pos} is_showing={*is_showing}>
            <ul>
                {for buttons}
            </ul>
        </ContextMenu>
    }
}
