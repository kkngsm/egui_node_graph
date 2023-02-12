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
use crate::state::{
    AnyParameterId, Graph, MousePosOnNode, NodeFinder, NodeId, NodeTemplateIter, NodeTemplateTrait,
    PortsData, WidgetValueTrait,
};
use crate::utils::{get_center, get_near, get_offset_from_current_target};
use crate::Vec2;
use glam::vec2;
use gloo::events::EventListener;
use gloo::utils::window;
use slotmap::SecondaryMap;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use yew::prelude::*;

pub type PortsRef = Rc<RefCell<PortsData<NodeRef>>>;
#[derive(Default)]
pub struct GraphRef(NodeRef);

const NEAR_THRESHOLD: f32 = 100.0;
/// Basic GraphEditor components
/// The following limitations apply
/// - NodeFinder is the default
/// - UserState must implement PartialEq
/// If you want a broader implementation, you may want to define your own components
#[derive(Default)]
pub struct BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
where
    NodeData: 'static,
    DataType: 'static,
    ValueType: 'static,
    NodeTemplate: 'static,
    UserState: 'static,
{
    graph: Graph<NodeData, DataType, ValueType>,
    //TODO
    // /// Nodes are drawn in this order. Draw order is important because nodes
    // /// that are drawn last are on top.
    // pub node_order: Vec<NodeId>,
    /// An ongoing connection interaction: The mouse has dragged away from a
    /// port and the user is holding the click
    connection_in_progress: Option<(AnyParameterId, Vec2)>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    selected_nodes: HashSet<NodeId>,

    // /// The mouse drag start position for an ongoing box selection.
    // pub ongoing_box_selection: Option<crate::Vec2>,
    /// The position of each node.
    node_positions: SecondaryMap<NodeId, crate::Vec2>,

    /// The position of each port.
    port_refs: PortsRef,

    /// The node finder is used to create new nodes.
    node_finder: NodeFinder,

    // /// The panning of the graph viewport.
    // pub pan_zoom: PanZoom,
    ///
    mouse_on_node: Option<MousePosOnNode>,

    graph_ref: GraphRef,

    _drag_event: Option<[EventListener; 2]>,

    _user_state: PhantomData<fn() -> UserState>,
    _template: PhantomData<fn() -> NodeTemplate>,
}
#[derive(Debug, Clone)]
pub enum GraphMessage<NodeTemplate> {
    SelectNode {
        id: NodeId,
        shift_key: bool,
    },

    DragStartPort(AnyParameterId),
    DragStartNode {
        data: MousePosOnNode,
        shift_key: bool,
    },
    Dragging(Vec2),
    DragEnd,

    // NodeFinder Event
    OpenNodeFinder(Vec2),
    CreateNode(NodeTemplate),

    BackgroundClick,

    None,
}

/// Props for [`BasicGraphEditor`]
#[derive(Properties, PartialEq)]
pub struct BasicGraphEditorProps<UserState: PartialEq> {
    pub user_state: Rc<RefCell<UserState>>,
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState> Component
    for BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
where
    UserState: PartialEq,
    NodeTemplate: NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + NodeTemplateIter<Item = NodeTemplate>
        + PartialEq
        + Copy
        + Debug,
    DataType: Display + PartialEq + Clone,
    ValueType: WidgetValueTrait<UserState = UserState, NodeData = NodeData> + Clone,
{
    type Message = GraphMessage<NodeTemplate>;
    type Properties = BasicGraphEditorProps<UserState>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            graph: Default::default(),
            selected_nodes: Default::default(),
            connection_in_progress: Default::default(),
            node_positions: Default::default(),
            port_refs: Default::default(),
            node_finder: Default::default(),
            mouse_on_node: Default::default(),
            graph_ref: Default::default(),
            _drag_event: Default::default(),
            _user_state: PhantomData,
            _template: PhantomData,
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", &msg);
        let BasicGraphEditorProps { user_state } = ctx.props();
        let user_state = &mut *user_state.borrow_mut();
        match msg {
            GraphMessage::SelectNode { id, shift_key } => {
                if !shift_key {
                    self.selected_nodes.clear();
                }
                self.selected_nodes.insert(id);
                true
            }
            GraphMessage::DragStartNode { data, shift_key } => {
                self.set_drag_event(ctx.link().callback(|msg| msg));

                if !shift_key {
                    self.selected_nodes.clear();
                }
                self.selected_nodes.insert(data.id);
                self.mouse_on_node = Some(data);
                false
            }
            GraphMessage::DragStartPort(id) => {
                self.set_drag_event(ctx.link().callback(|msg| msg));
                if let AnyParameterId::Input(input_id) = id {
                    if self.graph.connections().contains_key(input_id) {
                        let output_id = self.graph.connections_mut().remove(input_id).unwrap();
                        self.connection_in_progress = Some((
                            output_id.into(),
                            self.port_refs
                                .borrow()
                                .output
                                .get(output_id)
                                .map(get_center)
                                .unwrap_or_default(),
                        ));
                    } else {
                        self.connection_in_progress = Some((
                            id,
                            self.port_refs
                                .borrow()
                                .get(id)
                                .map(get_center)
                                .unwrap_or_default(),
                        ));
                    }
                } else {
                    self.connection_in_progress = Some((
                        id,
                        self.port_refs
                            .borrow()
                            .get(id)
                            .map(get_center)
                            .unwrap_or_default(),
                    ));
                }
                false
            }
            GraphMessage::Dragging(mouse_pos) => {
                let offset = self.graph_ref.get_offset().unwrap_or_default();
                let global_mouse_pos = mouse_pos + offset;

                // Connecting to p ort
                if let Some((id, p)) = self.connection_in_progress.as_mut() {
                    // snap to port
                    let nearest_port_pos = match *id {
                        AnyParameterId::Input(input_id) => self
                            .port_refs
                            .borrow()
                            .output
                            .iter()
                            .find_map(get_near(global_mouse_pos, NEAR_THRESHOLD))
                            .map(|(output_id, pos)| (output_id, input_id, pos)),
                        AnyParameterId::Output(output_id) => self
                            .port_refs
                            .borrow()
                            .input
                            .iter()
                            .find_map(get_near(global_mouse_pos, NEAR_THRESHOLD))
                            .map(|(input_id, pos)| (output_id, input_id, pos)),
                    }
                    .and_then(|(output, input, pos)| {
                        self.graph.param_typ_eq(output, input).then(|| pos - offset)
                    });

                    *p = nearest_port_pos.unwrap_or(mouse_pos);
                    true

                // Dragging node
                } else if let Some(MousePosOnNode { id, gap }) = self.mouse_on_node {
                    let pos = mouse_pos - gap;
                    let selected_pos = self.node_positions[id];
                    let drag_delta = pos - selected_pos;
                    for id in &self.selected_nodes {
                        let id = *id;
                        self.node_positions[id] += drag_delta;
                    }
                    true
                } else {
                    false
                }
            }
            GraphMessage::DragEnd => {
                self._drag_event = None;
                self.mouse_on_node = None;

                // Connect to Port
                if let Some((id, pos)) = self.connection_in_progress.take() {
                    let offset = self.graph_ref.get_offset().unwrap_or_default();
                    let global_pos = pos + offset;
                    let nearest_port = match id {
                        AnyParameterId::Input(input) => self
                            .port_refs
                            .borrow()
                            .output
                            .iter()
                            .find_map(get_near(global_pos, NEAR_THRESHOLD))
                            .map(|(output, _)| (input, output)),
                        AnyParameterId::Output(output) => self
                            .port_refs
                            .borrow()
                            .input
                            .iter()
                            .find_map(get_near(global_pos, NEAR_THRESHOLD))
                            .map(|(input, _)| (input, output)),
                    };
                    if let Some((input, output)) = nearest_port {
                        if self.graph.param_typ_eq(output, input) {
                            self.graph.connections_mut().insert(input, output);
                        }
                    }
                }
                true
            }
            GraphMessage::CreateNode(template) => {
                let new_node = self.graph.add_node(
                    template.node_graph_label(user_state),
                    template.user_data(user_state),
                    |graph, node_id| template.build_node(graph, user_state, node_id),
                );
                self.node_positions.insert(new_node, self.node_finder.pos);
                self.selected_nodes.insert(new_node);

                let node = &self.graph[new_node];
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
            GraphMessage::BackgroundClick => {
                let mut changed = false;
                let is_showing = &mut self.node_finder.is_showing;
                changed |= if *is_showing {
                    *is_showing = false;
                    true
                } else {
                    false
                };

                changed |= if self.selected_nodes.is_empty() {
                    false
                } else {
                    self.selected_nodes.clear();
                    true
                };
                changed
            }
            GraphMessage::None => false,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        use GraphMessage::*;
        let BasicGraphEditorProps { user_state } = ctx.props();
        let nodes = self.graph.nodes.keys().map(|id| {
            let node_event = ctx.link().callback(move |e| match e {
                NodeEvent::Select { shift_key } => SelectNode { id, shift_key },
                NodeEvent::DragStart { gap, shift_key } => DragStartNode {
                    data: MousePosOnNode { id, gap },
                    shift_key,
                },
                NodeEvent::Port(PortEvent::MouseDown(id)) => DragStartPort(id),
            });
            html! {<Node<NodeData, DataType, ValueType, UserState>
                key={id.to_string()}
                data={self.graph[id].clone()}
                pos={self.node_positions[id]}
                is_selected={self.selected_nodes.contains(&id)}
                onevent={node_event}
                {user_state}
                input_params={self.graph.inputs.clone()}
                output_params={self.graph.outputs.clone()}
                connections={self.graph.connections.clone()}
                ports_ref={self.port_refs.clone()}
            />}
        });

        let background_event = ctx.link().callback(|e: BackgroundEvent| match e {
            BackgroundEvent::ContextMenu(pos) => OpenNodeFinder(pos),
            BackgroundEvent::Click(_) => BackgroundClick,
        });
        let edges = self.graph_ref.get_offset().map(|offset|{
            let connection_in_progress = self.connection_in_progress.map(|(id, pos)| {
                let (output, input, typ) = match id {
                    AnyParameterId::Input(id) => (
                        pos,
                        self.port_refs
                            .borrow()
                            .input
                            .get(id)
                            .map(get_center)
                            .unwrap_or_default() - offset,
                        self.graph.inputs.borrow()[id].typ.clone(),
                    ),
                    AnyParameterId::Output(id) => (
                        self.port_refs
                            .borrow()
                            .output
                            .get(id)
                            .map(get_center)
                            .unwrap_or_default() - offset,
                        pos,
                        self.graph.outputs.borrow()[id].typ.clone(),
                    ),
                };
                html! {
                    <Edge<DataType> {output} {input} {typ}/>
                }
            });

            let connections = self.graph.connections();
            let edges = connections.iter().map(|(input, output)| {
                let typ = self.graph.input(input).typ.clone();
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
                html! {<Edge<DataType> key={output.to_string()} output={output_pos-offset} input={input_pos-offset} {typ} />}
            });

            html! {
                <>
                {for edges}
                {connection_in_progress}
                </>
            }
        });

        html! {
            <GraphArea
                node_ref={self.graph_ref.clone()}
                onevent={background_event}
            >
            {for nodes}
            {edges}
            <BasicNodeFinder<NodeTemplate, UserState>
                is_showing={self.node_finder.is_showing}
                pos={self.node_finder.pos}
                user_state={user_state.clone()}
                onevent={ctx.link().callback(|t| CreateNode(t))}
            />
            </GraphArea>
        }
    }
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState>
    BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
{
    pub fn set_drag_event(&mut self, onevent: Callback<GraphMessage<NodeTemplate>>) {
        let document = window().document().unwrap();

        self._drag_event = Some([
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
    pub user_state: Rc<RefCell<UserState>>,
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
    let user_state = &mut *user_state.borrow_mut();

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
