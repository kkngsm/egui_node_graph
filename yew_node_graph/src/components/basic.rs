use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::components::contextmenu::ContextMenu;
use crate::components::edge::Edge;
use crate::components::graph::{BackgroundEvent, GraphArea};
use crate::components::node::{Node, NodeEvent};
use crate::components::port::PortEvent;
use crate::components::select_box::SelectBox;
use crate::state::basic::BasicGraphEditorState;
use crate::state::{
    AnyParameterId, DragState, InputId, NodeDataTrait, NodeId, NodeTemplateIter, NodeTemplateTrait,
    OutputId, UserResponseTrait, WidgetValueTrait,
};
use crate::utils::{get_center, get_offset, get_offset_from_current_target};
use crate::Vec2;

use gloo_events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

/// Props for [`BasicGraphEditor`]
#[derive(Properties)]
pub struct BasicGraphEditorProps<
    NodeData,
    DataType,
    ValueType,
    NodeTemplate,
    UserState,
    UserResponse,
> where
    UserState: PartialEq,
{
    pub user_state: UserState,
    pub graph_editor_state:
        Rc<RefCell<BasicGraphEditorState<NodeData, DataType, ValueType, NodeTemplate>>>,
    #[prop_or_default]
    pub callback: Callback<BasicGraphEditorResponse<NodeData, UserResponse>>,
}
impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse> PartialEq
    for BasicGraphEditorProps<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
where
    UserState: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.user_state == other.user_state
            && Rc::ptr_eq(&self.graph_editor_state, &other.graph_editor_state)
            && self.callback == other.callback
    }
}

#[function_component(BasicGraphEditor)]
pub fn basic_graph_editor<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>(
    BasicGraphEditorProps {
        user_state,
        graph_editor_state,
        callback,
    }: &BasicGraphEditorProps<
        NodeData,
        DataType,
        ValueType,
        NodeTemplate,
        UserState,
        UserResponse,
    >,
) -> Html
where
    NodeData: NodeDataTrait<
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
            Response = UserResponse,
        > + Clone
        + 'static,
    DataType: Display + PartialEq + Clone + 'static,
    ValueType: WidgetValueTrait<NodeData = NodeData, UserState = UserState, Response = UserResponse>
        + Clone
        + 'static,
    UserState: Clone + PartialEq + 'static,
    UserResponse: UserResponseTrait + 'static,
    NodeTemplate: Copy
        + NodeTemplateIter<Item = NodeTemplate>
        + NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + PartialEq
        + Clone
        + 'static,
{
    let event_listener = use_mut_ref(|| Option::<[gloo_events::EventListener; 2]>::None);
    let updater = use_force_update();

    let node_callback = Callback::from({
        let callback = callback.clone();
        let state = graph_editor_state.clone();
        let event_listener = event_listener.clone();
        let updater = updater.clone();
        move |(id, n)| match n {
            NodeEvent::Delete => {
                let (node, disconnected) = state.borrow_mut().delete_node(id);
                callback.emit(BasicGraphEditorResponse::DeleteNode { node_id: id, node });
                for (input, output) in disconnected {
                    callback.emit(BasicGraphEditorResponse::DisconnectEvent { output, input });
                }
                updater.force_update();
            }
            NodeEvent::MouseDown { shift, shift_key } => {
                {
                    let mut state = state.borrow_mut();
                    state.selection(id, shift_key);
                    callback.emit(BasicGraphEditorResponse::SelectNode(id));
                    state.start_moving_node(id, shift, shift_key);
                }
                updater.force_update();
                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let callback = callback.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let mouse_pos = get_offset_from_current_target(e);
                            let mut state = state.borrow_mut();
                            let drag_delta = state.move_node(mouse_pos).unwrap();
                            for node in state.selected_nodes.iter().copied() {
                                callback
                                    .emit(BasicGraphEditorResponse::MoveNode { node, drag_delta });
                            }
                            updater.force_update();
                        }
                    },
                    {
                        let event_listener = event_listener.clone();
                        let updater = updater.clone();
                        let state = state.clone();
                        move |_| {
                            state.borrow_mut().end_moving_node();
                            event_listener.borrow_mut().take();
                            updater.force_update();
                        }
                    },
                );
            }
        }
    });
    let port_callback = Callback::from({
        let callback = callback.clone();
        let state = graph_editor_state.clone();
        let event_listener = event_listener.clone();
        let updater = updater.clone();
        move |(id, p)| match p {
            PortEvent::MouseDown => {
                {
                    let mut state = state.borrow_mut();
                    if let Some((output, input)) = state.start_connection(id) {
                        callback.emit(BasicGraphEditorResponse::DisconnectEvent { output, input });
                        let node = match id {
                            AnyParameterId::Input(input) => state.graph[input].node,
                            AnyParameterId::Output(output) => state.graph[output].node,
                        };
                        callback.emit(BasicGraphEditorResponse::ConnectEventStarted(node, id))
                    } else {
                        match id {
                            AnyParameterId::Output(output) => {
                                callback.emit(BasicGraphEditorResponse::ConnectEventStarted(
                                    state.graph[output].node,
                                    id,
                                ))
                            }
                            AnyParameterId::Input(input) => {
                                callback.emit(BasicGraphEditorResponse::ConnectEventStarted(
                                    state.graph[input].node,
                                    id,
                                ))
                            }
                        }
                    }
                    updater.force_update();
                }
                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let pos = get_offset_from_current_target(e);
                            state.borrow_mut().move_connection(pos);
                            updater.force_update();
                        }
                    },
                    {
                        let event_listener = event_listener.clone();
                        let callback = callback.clone();
                        let state = state.clone();
                        let updater = updater.clone();
                        move |_| {
                            if let Some((output, input)) = state.borrow_mut().end_connection() {
                                callback.emit(BasicGraphEditorResponse::ConnectEventEnded {
                                    output,
                                    input,
                                });
                            }
                            event_listener.borrow_mut().take();
                            updater.force_update();
                        }
                    },
                );
            }
            PortEvent::MouseEnter => {
                let BasicGraphEditorState {
                    graph, drag_state, ..
                } = &mut *state.borrow_mut();
                if let Some(DragState::ConnectPort(c)) = drag_state.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| graph.param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        c.to_id(id);
                        updater.force_update();
                    }
                }
            }
            PortEvent::MouseLeave => {
                let BasicGraphEditorState {
                    graph,
                    drag_state,
                    port_refs,
                    graph_ref,
                    ..
                } = &mut *state.borrow_mut();
                if let Some(DragState::ConnectPort(c)) = drag_state.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| graph.param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        let offset = get_offset(graph_ref);
                        let port_pos = port_refs
                            .borrow()
                            .get(id)
                            .and_then(get_center)
                            .zip(offset)
                            .map(|(p, o)| p - o)
                            .unwrap_or_default();
                        c.to_pos(port_pos);
                        updater.force_update();
                    }
                }
            }
        }
    });
    let background_event = Callback::from({
        let state = graph_editor_state.clone();
        let event_listener = event_listener;
        let updater = updater.clone();
        move |b: BackgroundEvent| match b {
            BackgroundEvent::ContextMenu(pos) => {
                let mut state = state.borrow_mut();
                state.node_finder.is_showing = true;
                state.node_finder.pos = pos;
                updater.force_update();
            }
            BackgroundEvent::MouseDown {
                button,
                pos,
                is_shift_key_pressed,
            } if button != 2 => {
                {
                    let mut state = state.borrow_mut();
                    state.node_finder.is_showing = false;
                    if !is_shift_key_pressed {
                        Rc::make_mut(&mut state.selected_nodes).clear();
                    }
                    state.start_select_box(pos);
                }
                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let end = get_offset_from_current_target(e);
                            state.borrow_mut().scale_select_box(end);
                            updater.force_update();
                        }
                    },
                    {
                        let event_listener = event_listener.clone();
                        let updater = updater.clone();
                        let state = state.clone();
                        move |_| {
                            state.borrow_mut().end_select_box();
                            event_listener.borrow_mut().take();
                            updater.force_update();
                        }
                    },
                );
            }
            _ => (),
        }
    });
    let finder_callback = Callback::from({
        let callback = callback.clone();
        let state = graph_editor_state.clone();
        let user_state = user_state.clone();
        let updater = updater.clone();
        move |t: NodeTemplate| {
            let id = state.borrow_mut().create_node(t, &user_state);
            callback.emit(BasicGraphEditorResponse::CreatedNode(id));
            updater.force_update();
        }
    });
    let user_callback = Callback::from({
        let callback = callback.clone();
        move |u: UserResponse| {
            callback.emit(BasicGraphEditorResponse::User(u));
            updater.force_update();
        }
    });

    let state = graph_editor_state.borrow();
    let graph = &state.graph;
    let nodes = graph.nodes.keys().map(|id| {
        let user_state = user_state.to_owned();
        html! {<Node<NodeData, DataType, ValueType, UserState, UserResponse>
            key={id.to_string()}
            data={graph[id].clone()}
            pos={state.node_positions[id]}
            is_selected={state.selected_nodes.contains(&id)}
            node_callback={node_callback.clone()}
            port_callback={port_callback.clone()}
            user_callback={user_callback.clone()}
            user_state={user_state}
            graph={state.graph.clone()}
            ports_ref={state.port_refs.clone()}
        />}
    });

    let connection_in_progress = state.connection_in_progress().map(|(output, input, typ)| {
        html! {
            <Edge<DataType> {output} {input} {typ}/>
        }
    });
    let select_box = state.select_box().map(|(start, end)| {
        html! {
            <SelectBox {start} {end} />
        }
    });
    let edges = get_offset(&state.graph_ref).map(|offset|{
            let edges = graph.connections.iter().map(|(input, output)| {
                let typ = graph.inputs[input].typ.clone();
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
                </>
            }
        });

    html! {
        <GraphArea
            node_ref={state.graph_ref.clone()}
            onevent={background_event}
        >
        {for nodes}
        {edges}
        {connection_in_progress}
        {select_box}
        <BasicNodeFinder<NodeTemplate, UserState>
            is_showing={state.node_finder.is_showing}
            pos={state.node_finder.pos}
            user_state={user_state.to_owned()}
            onevent={finder_callback}
        />
        </GraphArea>
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

pub enum BasicGraphEditorResponse<NodeData, UserResponse> {
    ConnectEventStarted(NodeId, AnyParameterId),
    ConnectEventEnded {
        output: OutputId,
        input: InputId,
    },
    CreatedNode(NodeId),
    SelectNode(NodeId),

    DeleteNode {
        node_id: NodeId,
        node: Rc<crate::state::Node<NodeData>>,
    },
    DisconnectEvent {
        output: OutputId,
        input: InputId,
    },
    /// Emitted when a node is interacted with, and should be raised
    RaiseNode(NodeId),
    MoveNode {
        node: NodeId,
        drag_delta: Vec2,
    },
    User(UserResponse),
}

fn set_drag_event(
    graph_ref: &NodeRef,
    mousemove: impl FnMut(&Event) + 'static,
    mouseup: impl FnMut(&Event) + 'static,
) -> Option<[EventListener; 2]> {
    graph_ref.cast::<web_sys::Element>().map(|element| {
        let document = web_sys::window().unwrap().document().unwrap();
        [
            EventListener::new(&document, "mouseup", mouseup),
            EventListener::new(&element, "mousemove", mousemove),
        ]
    })
}
