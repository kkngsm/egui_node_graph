use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::components::edge::Edge;
use crate::components::graph_area::{BackgroundEvent, GraphArea};
use crate::components::node::{Node, NodeEvent};
use crate::components::port::PortEvent;
use crate::components::select_box::SelectBox;
use crate::components::NodeFinder;
use crate::state::graph_editor::GraphEditorState;
use crate::state::{
    AnyParameterId, DragState, InputId, NodeDataTrait, NodeId, NodeTemplateIter, NodeTemplateTrait,
    OutputId, UserResponseTrait, WidgetValueTrait,
};
use crate::utils::{get_center, get_mouse_pos_from_current_target, get_offset};
use crate::Vec2;

use gloo_events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

/// Properties for  [`GraphEditor`]
#[derive(Properties)]
pub struct GraphEditorProps<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
where
    UserState: PartialEq,
{
    pub user_state: UserState,
    pub graph_editor_state:
        Rc<RefCell<GraphEditorState<NodeData, DataType, ValueType, NodeTemplate>>>,
    #[prop_or_default]
    pub callback: Callback<GraphEditorResponse<NodeData, UserResponse>>,
}
impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse> PartialEq
    for GraphEditorProps<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
where
    UserState: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.user_state == other.user_state
            && Rc::ptr_eq(&self.graph_editor_state, &other.graph_editor_state)
            && self.callback == other.callback
    }
}

#[function_component(GraphEditor)]
pub fn graph_editor<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>(
    GraphEditorProps {
        user_state,
        graph_editor_state,
        callback,
    }: &GraphEditorProps<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>,
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
                callback.emit(GraphEditorResponse::DeleteNode { node_id: id, node });
                for (input, output) in disconnected {
                    callback.emit(GraphEditorResponse::DisconnectEvent { output, input });
                }
                updater.force_update();
            }
            NodeEvent::MouseDown { shift, shift_key } => {
                {
                    let mut state = state.borrow_mut();
                    state.selection(id, shift_key);
                    state.start_moving_node(id, shift);
                }
                callback.emit(GraphEditorResponse::SelectNode(id));
                updater.force_update();
                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let callback = callback.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let mouse_pos = get_mouse_pos_from_current_target(e);

                            let drag_delta = state.borrow_mut().move_node(mouse_pos).unwrap();
                            for node in state
                                .borrow()
                                .selected_nodes
                                .iter()
                                .copied()
                                .collect::<Vec<_>>()
                            {
                                callback.emit(GraphEditorResponse::MoveNode { node, drag_delta });
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
                let connection = state.borrow_mut().start_connection(id);
                if let Some((output, input)) = connection {
                    callback.emit(GraphEditorResponse::DisconnectEvent { output, input });
                    let node = match id {
                        AnyParameterId::Input(input) => state.borrow().graph[input].node,
                        AnyParameterId::Output(output) => state.borrow().graph[output].node,
                    };
                    callback.emit(GraphEditorResponse::ConnectEventStarted(node, id))
                } else {
                    match id {
                        AnyParameterId::Output(output) => {
                            callback.emit(GraphEditorResponse::ConnectEventStarted(
                                state.borrow().graph[output].node,
                                id,
                            ))
                        }
                        AnyParameterId::Input(input) => {
                            callback.emit(GraphEditorResponse::ConnectEventStarted(
                                state.borrow().graph[input].node,
                                id,
                            ))
                        }
                    }
                }
                updater.force_update();

                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let pos = get_mouse_pos_from_current_target(e);
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
                            let connection = state.borrow_mut().end_connection();
                            if let Some((output, input)) = connection {
                                callback
                                    .emit(GraphEditorResponse::ConnectEventEnded { output, input });
                            }
                            event_listener.borrow_mut().take();
                            updater.force_update();
                        }
                    },
                );
            }
            PortEvent::MouseEnter => {
                let GraphEditorState {
                    graph, drag_state, ..
                } = &mut *state.borrow_mut();
                if let Some(DragState::ConnectPort(c)) = drag_state.as_mut() {
                    let is_able_to_connect = c
                        .pair_with(id)
                        .map(|(output, input)| {
                            // Don't allow self-loops
                            graph[output].node != graph[input].node
                                && graph.param_typ_eq(output, input)
                        })
                        .unwrap_or_default();
                    if is_able_to_connect {
                        c.to_id(id);
                        updater.force_update();
                    }
                }
            }
            PortEvent::MouseLeave => {
                let GraphEditorState {
                    graph,
                    drag_state,
                    port_refs,
                    graph_ref,
                    ..
                } = &mut *state.borrow_mut();
                if let Some(DragState::ConnectPort(c)) = drag_state.as_mut() {
                    let is_able_to_connect = c
                        .pair_with(id)
                        .map(|(output, input)| {
                            // Don't allow self-loops
                            graph[output].node != graph[input].node
                                && graph.param_typ_eq(output, input)
                        })
                        .unwrap_or_default();
                    if is_able_to_connect {
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
            } if button == 0 => {
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
                            let end = get_mouse_pos_from_current_target(e);
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
            BackgroundEvent::MouseDown { button, pos, .. } if button == 1 => {
                {
                    log::debug!("pan");
                    let mut state = state.borrow_mut();
                    state.node_finder.is_showing = false;
                    state.drag_state = Some(DragState::Pan { prev_pos: pos })
                }
                *event_listener.borrow_mut() = set_drag_event(
                    &state.borrow().graph_ref,
                    {
                        let updater = updater.clone();
                        let state = state.clone();
                        move |e| {
                            let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                            let view_pos = get_mouse_pos_from_current_target(e);
                            let GraphEditorState {
                                pan_zoom,
                                drag_state,
                                ..
                            } = &mut *state.borrow_mut();
                            if let Some(DragState::Pan { prev_pos }) = drag_state {
                                let delta = view_pos - *prev_pos;
                                pan_zoom.pan += delta;
                                *prev_pos = view_pos;
                            }
                            updater.force_update();
                        }
                    },
                    {
                        let event_listener = event_listener.clone();
                        let updater = updater.clone();
                        move |_| {
                            event_listener.borrow_mut().take();
                            updater.force_update();
                        }
                    },
                );
            }
            BackgroundEvent::Wheel { delta_y, pos } => {
                let mut state = state.borrow_mut();
                let zoom = state.pan_zoom.zoom
                    + if delta_y.is_sign_negative() {
                        0.1
                    } else {
                        -0.1
                    };
                if zoom >= 0.1 && zoom <= 2.0 {
                    let logical_pos = state.pan_zoom.screen2logical(pos);
                    let zoom = zoom / state.pan_zoom.zoom;
                    state.pan_zoom.zoom_to_pos(zoom, logical_pos);
                }
                updater.force_update();
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
            let id = {
                let mut state = state.borrow_mut();
                state.node_finder.is_showing = false;
                state.create_node(t, &user_state)
            };
            callback.emit(GraphEditorResponse::CreatedNode(id));
            updater.force_update();
        }
    });
    let user_callback = Callback::from({
        let callback = callback.clone();
        move |u: UserResponse| {
            callback.emit(GraphEditorResponse::User(u));
            updater.force_update();
        }
    });

    let graph = graph_editor_state.borrow().graph.clone();
    let nodes = graph.nodes.keys().map(|id| {
        let state = graph_editor_state.borrow();
        let logical = state.node_positions[id];
        let pos = state.pan_zoom.logical2screen(logical);
        let user_state = user_state.to_owned();
        html! {<Node<NodeData, DataType, ValueType, UserState, UserResponse>
            key={id.to_string()}
            data={graph[id].clone()}
            {pos}
            is_selected={state.selected_nodes.contains(&id)}
            node_callback={node_callback.clone()}
            port_callback={port_callback.clone()}
            user_callback={user_callback.clone()}
            user_state={user_state}
            graph={graph.clone()}
            ports_ref={state.port_refs.clone()}
        />}
    });

    let connection_in_progress =
        graph_editor_state
            .borrow()
            .connection_in_progress()
            .map(|(output, input, typ)| {
                html! {
                    <Edge<DataType> {output} {input} {typ}/>
                }
            });
    let select_box = graph_editor_state
        .borrow()
        .select_box()
        .map(|(start, end)| {
            html! {
                <SelectBox {start} {end} />
            }
        });
    let edges = get_offset(&graph_editor_state
        .borrow().graph_ref).map(|offset|{
            let edges = graph.connections.iter().map(|(input, output)| {
                let typ = graph.inputs[input].typ.clone();
                let port_refs = graph_editor_state
                .borrow().port_refs.clone();
                let output_pos = port_refs
                    .borrow()
                    .output
                    .get(*output)
                    .and_then(get_center)
                    .map(|p| p - offset)
                    .unwrap_or_default();
                let input_pos =
                    port_refs.borrow()
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
        <>
        <GraphArea
            node_ref={graph_editor_state.borrow().graph_ref.clone()}
            onevent={background_event}
        >
        {for nodes}
        {edges}
        {connection_in_progress}
        {select_box}
        </GraphArea>
        <NodeFinder<NodeTemplate, UserState>
            is_showing={graph_editor_state.borrow().node_finder.is_showing}
            pos={graph_editor_state.borrow().node_finder.pos}
            user_state={user_state.to_owned()}
            onevent={finder_callback}
        />
        </>
    }
}

#[derive(Debug, Clone)]
pub enum GraphEditorResponse<NodeData, UserResponse> {
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
