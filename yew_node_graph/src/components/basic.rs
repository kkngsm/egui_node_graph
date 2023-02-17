use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use crate::components::contextmenu::ContextMenu;
use crate::components::edge::Edge;
use crate::components::graph::{BackgroundEvent, GraphArea};
use crate::components::node::{Node, NodeEvent};
use crate::components::port::PortEvent;
use crate::components::select_box::SelectBox;
use crate::state::{
    AnyParameterId, ConnectTo, ConnectionInProgress, DragState, Graph, NodeDataTrait, NodeFinder,
    NodeId, NodeTemplateIter, NodeTemplateTrait, PortRefs, UserResponseTrait, WidgetValueTrait,
};
use crate::utils::{get_center, get_offset_from_current_target};
use crate::Vec2;
use glam::vec2;
use gloo::events::EventListener;
use gloo::utils::window;
use slotmap::SecondaryMap;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

#[derive(Default)]
pub struct GraphRef(NodeRef);

/// Basic GraphEditor components
/// The following limitations apply
/// - NodeFinder is the default
/// - UserState must implement PartialEq
/// If you want a broader implementation, you may want to define your own components
#[derive(Default)]
pub struct BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate>
where
    NodeData: 'static,
    DataType: 'static,
    ValueType: 'static,
{
    graph: Rc<RefCell<Graph<NodeData, DataType, ValueType>>>,
    //TODO
    // /// Nodes are drawn in this order. Draw order is important because nodes
    // /// that are drawn last are on top.
    // pub node_order: Vec<NodeId>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    selected_nodes: HashSet<NodeId>,

    /// The position of each node.
    node_positions: SecondaryMap<NodeId, crate::Vec2>,

    port_refs: PortRefs,

    node_finder: NodeFinder,

    // /// The panning of the graph viewport.
    // pub pan_zoom: PanZoom,
    ///
    graph_ref: GraphRef,

    drag_event: Option<DragState>,
    _drag_event_listener: Option<[EventListener; 2]>,

    _template: PhantomData<fn() -> NodeTemplate>,
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
    NodeData: NodeDataTrait<
        DataType = DataType,
        ValueType = ValueType,
        UserState = UserState,
        Response = UserResponse,
    >,
    UserState: Clone + PartialEq + 'static,
    NodeTemplate: NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + NodeTemplateIter<Item = NodeTemplate>
        + PartialEq
        + Copy
        + Debug
        + 'static,
    DataType: Display + PartialEq + Clone,
    ValueType: WidgetValueTrait<UserState = UserState, NodeData = NodeData, Response = UserResponse>
        + Clone,
    UserResponse: UserResponseTrait + 'static,
{
    type Message = GraphMessage<NodeTemplate, UserResponse>;
    type Properties = BasicGraphEditorProps<UserState, UserResponse>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            graph: Default::default(),
            selected_nodes: Default::default(),
            node_positions: Default::default(),
            port_refs: Default::default(),
            node_finder: Default::default(),
            graph_ref: Default::default(),
            drag_event: Default::default(),
            _drag_event_listener: Default::default(),
            _template: PhantomData,
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", &msg);
        let BasicGraphEditorProps {
            user_state,
            callback,
        } = ctx.props();
        match msg {
            GraphMessage::SelectNode { id, shift_key } => {
                if !self.selected_nodes.contains(&id) {
                    if !shift_key {
                        self.selected_nodes.clear();
                    }
                    self.selected_nodes.insert(id);
                }
                true
            }
            GraphMessage::DeleteNode { id } => {
                let (_node, _disc_events) = self.graph.borrow_mut().remove_node(id);
                true
            }
            GraphMessage::DragStartNode {
                id,
                shift,
                shift_key,
            } => {
                if self.selected_nodes.contains(&id) {
                    self.set_drag_event(ctx.link().callback(|msg| msg));
                    self.drag_event = Some(DragState::MoveNode {
                        id,
                        shift,
                        is_moved: false,
                        is_shift_key_pressed: shift_key,
                    });
                }
                false
            }
            GraphMessage::DragStartPort(id) => {
                self.set_drag_event(ctx.link().callback(|msg| msg));
                let pos = self
                    .port_refs
                    .borrow()
                    .get(id)
                    .map(get_center)
                    .unwrap_or_default();

                if let AnyParameterId::Input(input) = id {
                    if let Some(output) = self.graph.borrow_mut().connections.remove(input) {
                        self.drag_event = Some(DragState::ConnectPort((output, pos).into()));
                    } else {
                        self.drag_event = Some(DragState::ConnectPort((input, pos).into()));
                    }
                } else {
                    self.drag_event = Some(DragState::ConnectPort((id, pos).into()));
                }

                false
            }
            GraphMessage::DragStartBackground {
                pos,
                is_shift_key_pressed,
            } => {
                self.set_drag_event(ctx.link().callback(|msg| msg));
                if !is_shift_key_pressed {
                    self.selected_nodes.clear();
                }
                self.drag_event = Some(DragState::SelectBox {
                    start: pos,
                    end: pos,
                });
                true
            }
            GraphMessage::Dragging(mouse_pos) => {
                match self.drag_event.as_mut() {
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
                        let selected_pos = self.node_positions[*id];
                        let drag_delta = pos - selected_pos;
                        for id in &self.selected_nodes {
                            let id = *id;
                            self.node_positions[id] += drag_delta;
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
                if let Some(DragState::ConnectPort(c)) = self.drag_event.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| self.graph.borrow().param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        c.to_id(id);
                    }
                }

                true
            }
            GraphMessage::LeavePort(id) => {
                if let Some(DragState::ConnectPort(c)) = self.drag_event.as_mut() {
                    let typ_eq = c
                        .pair_with(id)
                        .map(|(output, input)| self.graph.borrow().param_typ_eq(output, input))
                        .unwrap_or_default();
                    if typ_eq {
                        let offset = self.graph_ref.get_offset().unwrap_or_default();
                        let port_pos = self
                            .port_refs
                            .borrow()
                            .get(id)
                            .map(|n| get_center(n) - offset)
                            .unwrap_or_default();
                        c.to_pos(port_pos);
                    }
                }
                false
            }
            GraphMessage::DragEnd => {
                self._drag_event_listener = None;
                self.node_finder.is_showing = false;
                let mut graph = self.graph.borrow_mut();
                match self.drag_event.take() {
                    Some(DragState::SelectBox { start, end }) => {
                        let min = start.min(end);
                        let max = start.max(end);
                        for id in self.node_positions.iter().flat_map(|(id, pos)| {
                            (min.cmplt(*pos).all() && pos.cmplt(max).all()).then_some(id)
                        }) {
                            self.selected_nodes.insert(id);
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
                                if !self.selected_nodes.remove(&id) {
                                    self.selected_nodes.insert(id);
                                }
                            } else {
                                self.selected_nodes.clear();
                                self.selected_nodes.insert(id);
                            }
                        }
                    }
                    _ => (),
                }

                true
            }
            GraphMessage::CreateNode(template) => {
                let new_node = self.graph.borrow_mut().add_node(
                    template.node_graph_label(user_state),
                    template.user_data(user_state),
                    |graph, node_id| template.build_node(graph, user_state, node_id),
                );
                self.node_positions.insert(new_node, self.node_finder.pos);
                self.selected_nodes.insert(new_node);

                let node = &self.graph.borrow()[new_node];
                for input in node.input_ids() {
                    self.port_refs
                        .borrow_mut()
                        .input
                        .insert(input, Default::default());
                }
                for output in node.output_ids() {
                    self.port_refs
                        .borrow_mut()
                        .output
                        .insert(output, Default::default());
                }
                true
            }
            GraphMessage::OpenNodeFinder(pos) => {
                self.node_finder.is_showing = true;
                self.node_finder.pos = pos;
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
        let graph = self.graph.borrow();
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
                pos={self.node_positions[id]}
                is_selected={self.selected_nodes.contains(&id)}
                onevent={node_event}
                user_state={user_state}
                graph={self.graph.clone()}
                ports_ref={self.port_refs.clone()}
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

        let edges_and_drag = self.graph_ref.get_offset().map(|offset|{
            let port_refs = self.port_refs.borrow();
            let drag = match &self.drag_event {
                Some(DragState::ConnectPort(c)) => {
                    let connection_in_progress = match c {
                        ConnectionInProgress::FromInput {
                            src: from,
                            dest: to,
                        } => Some((
                            to.map_pos(|id| {
                                port_refs
                                    .output
                                    .get(*id)
                                    .map(|n| get_center(n) - offset)
                                    .unwrap_or_default()
                            }),
                            port_refs
                                .input
                                .get(*from)
                                .map(|n| get_center(n) - offset)
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
                                .map(|n| get_center(n) - offset)
                                .unwrap_or_default(),
                            to.map_pos(|id| {
                                port_refs
                                    .input
                                    .get(*id)
                                    .map(|n| get_center(n) - offset)
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
            let graph = self.graph.borrow();
            let edges = graph.connections.iter().map(|(input, output)| {
                let typ = self.graph.borrow().inputs[input].typ.clone();
                let output_pos = self
                    .port_refs.borrow()
                    .output
                    .get(*output)
                    .map(get_center)
                    .unwrap_or_default();
                let input_pos = self
                    .port_refs.borrow()
                    .input
                    .get(input)
                    .map(get_center)
                    .unwrap_or_default();
                html! {<Edge<DataType> key={input.to_string()} output={output_pos-offset} input={input_pos-offset} {typ} />}
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
                node_ref={self.graph_ref.clone()}
                onevent={background_event}
            >
            {for nodes}
            {edges_and_drag}
            <BasicNodeFinder<NodeTemplate, UserState>
                is_showing={self.node_finder.is_showing}
                pos={self.node_finder.pos}
                user_state={user_state.to_owned()}
                onevent={ctx.link().callback(|t| CreateNode(t))}
            />
            </GraphArea>
        }
    }
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
    BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate>
where
    NodeData: NodeDataTrait<
        DataType = DataType,
        ValueType = ValueType,
        UserState = UserState,
        Response = UserResponse,
    >,
    NodeTemplate: NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + NodeTemplateIter<Item = NodeTemplate>
        + 'static,
    ValueType:
        WidgetValueTrait<UserState = UserState, NodeData = NodeData, Response = UserResponse>,
    UserResponse: 'static,
{
    pub fn set_drag_event(&mut self, onevent: Callback<GraphMessage<NodeTemplate, UserResponse>>) {
        let document = window().document().unwrap();

        self._drag_event_listener = Some([
            EventListener::new(&document, "mouseup", {
                let onevent = onevent.clone();
                move |_| onevent.emit(GraphMessage::DragEnd)
            }),
            EventListener::new(
                &self.graph_ref.cast::<web_sys::Element>().unwrap_throw(),
                "mousemove",
                {
                    move |e| {
                        let e = e.dyn_ref::<MouseEvent>().unwrap_throw();
                        onevent.emit(GraphMessage::Dragging(get_offset_from_current_target(e)))
                    }
                },
            ),
        ]);
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

impl GraphRef {
    pub fn get_offset(&self) -> Option<Vec2> {
        self.0.cast::<web_sys::Element>().map(|e| {
            let rect = e.get_bounding_client_rect();
            vec2(rect.x() as f32, rect.y() as f32)
        })
    }
}

impl Deref for GraphRef {
    type Target = NodeRef;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
