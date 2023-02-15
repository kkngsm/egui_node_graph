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
    AnyParameterId, ConnectTo, ConnectionInProgress, Graph, MousePosOnNode, NodeFinder, NodeId,
    NodeTemplateIter, NodeTemplateTrait, PortRefs, WidgetValueTrait,
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
    connection_in_progress: ConnectionInProgress,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    selected_nodes: HashSet<NodeId>,

    /// The mouse drag start position for an ongoing box selection.
    ongoing_box_selection: Option<(crate::Vec2, Vec2)>,

    /// The position of each node.
    node_positions: SecondaryMap<NodeId, crate::Vec2>,

    port_refs: PortRefs,

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
    DeleteNode {
        id: NodeId,
    },

    DragStartPort(AnyParameterId),
    DragStartNode {
        data: MousePosOnNode,
        shift_key: bool,
    },
    DragStartBackground(Vec2),

    Dragging(Vec2),
    EnterPort(AnyParameterId),
    LeavePort(AnyParameterId),
    DragEnd,

    // NodeFinder Event
    OpenNodeFinder(Vec2),
    CreateNode(NodeTemplate),

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
            ongoing_box_selection: Default::default(),
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
            GraphMessage::DeleteNode { id } => {
                let (_node, _disc_events) = self.graph.remove_node(id);
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
                let pos = self
                    .port_refs
                    .borrow()
                    .get(id)
                    .map(get_center)
                    .unwrap_or_default();

                if let AnyParameterId::Input(input) = id {
                    if let Some(output) = self.graph.connections.borrow_mut().remove(input) {
                        self.connection_in_progress = (output, pos).into();
                    } else {
                        self.connection_in_progress = (input, pos).into();
                    }
                } else {
                    self.connection_in_progress = (id, pos).into();
                }

                false
            }
            GraphMessage::DragStartBackground(pos) => {
                self.set_drag_event(ctx.link().callback(|msg| msg));
                let mut changed = false;

                changed |= if self.selected_nodes.is_empty() {
                    false
                } else {
                    self.selected_nodes.clear();
                    true
                };
                self.ongoing_box_selection = Some((pos, pos));
                changed
            }
            GraphMessage::Dragging(mouse_pos) => {
                // Connecting to port
                if let ConnectionInProgress::FromInput {
                    dest: ConnectTo::Pos(pos),
                    ..
                }
                | ConnectionInProgress::FromOutput {
                    dest: ConnectTo::Pos(pos),
                    ..
                } = &mut self.connection_in_progress
                {
                    *pos = mouse_pos;
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
                } else if let Some((_, pos)) = self.ongoing_box_selection.as_mut() {
                    *pos = mouse_pos;
                    true
                } else {
                    false
                }
            }
            GraphMessage::EnterPort(id) => {
                let typ_eq = self
                    .connection_in_progress
                    .connection_pair(id)
                    .map(|(output, input)| self.graph.param_typ_eq(output, input))
                    .unwrap_or_default();
                if typ_eq {
                    self.connection_in_progress.to_id(id);
                }
                true
            }
            GraphMessage::LeavePort(id) => {
                let typ_eq = self
                    .connection_in_progress
                    .connection_pair(id)
                    .map(|(output, input)| self.graph.param_typ_eq(output, input))
                    .unwrap_or_default();
                if typ_eq {
                    let offset = self.graph_ref.get_offset().unwrap_or_default();
                    let port_pos = self
                        .port_refs
                        .borrow()
                        .get(id)
                        .map(|n| get_center(n) - offset)
                        .unwrap_or_default();
                    self.connection_in_progress.to_pos(port_pos);
                }
                false
            }
            GraphMessage::DragEnd => {
                self._drag_event = None;
                self.mouse_on_node = None;
                self.node_finder.is_showing = false;

                // Connect to Port
                let connection = match self.connection_in_progress.take() {
                    ConnectionInProgress::FromInput {
                        src: input,
                        dest: ConnectTo::Id(output),
                    } => Some((output, input)),
                    ConnectionInProgress::FromOutput {
                        src: output,
                        dest: ConnectTo::Id(input),
                    } => Some((output, input)),
                    _ => None,
                };
                if let Some((output, input)) = connection {
                    if self.graph.param_typ_eq(output, input) {
                        self.graph.connections_mut().insert(input, output);
                    }
                }

                if let Some((start, end)) = self.ongoing_box_selection {
                    let min = start.min(end);
                    let max = start.max(end);
                    log::debug!("{}, {}", min, max);
                    for id in self.node_positions.iter().flat_map(|(id, pos)| {
                        (min.cmplt(*pos).all() && pos.cmplt(max).all()).then(|| id)
                    }) {
                        self.selected_nodes.insert(id);
                    }
                    self.ongoing_box_selection = None;
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
            GraphMessage::None => false,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        use GraphMessage::*;
        let BasicGraphEditorProps { user_state } = ctx.props();
        let nodes = self.graph.nodes.keys().map(|id| {
            let node_event = ctx.link().callback(move |e| match e {
                NodeEvent::Select { shift_key } => SelectNode { id, shift_key },
                NodeEvent::Delete => DeleteNode { id },
                NodeEvent::DragStart { gap, shift_key } => DragStartNode {
                    data: MousePosOnNode { id, gap },
                    shift_key,
                },
                NodeEvent::Port(PortEvent::MouseDown(id)) => DragStartPort(id),
                NodeEvent::Port(PortEvent::MouseEnter(id)) => EnterPort(id),
                NodeEvent::Port(PortEvent::MouseLeave(id)) => LeavePort(id),
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
            BackgroundEvent::MouseDown(button, pos) if button != 2 => DragStartBackground(pos),
            _ => None,
        });
        let edges = self.graph_ref.get_offset().map(|offset|{
            let port_refs = self.port_refs.borrow();
            let connection_in_progress =match &self.connection_in_progress{
                ConnectionInProgress::FromInput { src: from, dest: to } => Some((
                    to.map_pos(|id| port_refs.output.get(*id).map(|n| get_center(n)-offset).unwrap_or_default()),
                    port_refs.input.get(*from).map(|n| get_center(n)-offset).unwrap_or_default(),
                    self.graph.inputs.borrow()[*from].typ.clone()
                )),
                ConnectionInProgress::FromOutput { src: from, dest: to } => Some((
                    port_refs.output.get(*from).map(|n| get_center(n)-offset).unwrap_or_default(),
                    to.map_pos(|id| port_refs.input.get(*id).map(|n| get_center(n)-offset).unwrap_or_default()),
                    self.graph.outputs.borrow()[*from].typ.clone()
                )),
                ConnectionInProgress::None => Option::None,
            };
            let connection_in_progress = connection_in_progress.map(|(output, input, typ)|{
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
                html! {<Edge<DataType> key={input.to_string()} output={output_pos-offset} input={input_pos-offset} {typ} />}
            });

            html! {
                <>
                {for edges}
                {connection_in_progress}
                </>
            }
        });

        let select_box = self.ongoing_box_selection.map(|(start, end)| {
            html! {
                <SelectBox {start} {end} />
            }
        });

        html! {
            <GraphArea
                node_ref={self.graph_ref.clone()}
                onevent={background_event}
            >
            {for nodes}
            {edges}
            {select_box}
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
