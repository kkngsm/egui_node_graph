use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::components::*;
use crate::state::{
    Graph, MousePosOnNode, NodeFinder, NodeId, NodeTemplateIter, NodeTemplateTrait,
};
use crate::Vec2;
use gloo::events::EventListener;
use gloo::utils::window;
use slotmap::SecondaryMap;
use yew::prelude::*;
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
    pub graph: Graph<NodeData, DataType, ValueType>,
    //TODO
    // /// Nodes are drawn in this order. Draw order is important because nodes
    // /// that are drawn last are on top.
    // pub node_order: Vec<NodeId>,

    // /// An ongoing connection interaction: The mouse has dragged away from a
    // /// port and the user is holding the click
    // pub connection_in_progress: Option<(NodeId, AnyParameterId)>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    selected_nodes: HashSet<NodeId>,

    // /// The mouse drag start position for an ongoing box selection.
    // pub ongoing_box_selection: Option<crate::Vec2>,
    /// The position of each node.
    node_positions: SecondaryMap<NodeId, crate::Vec2>,

    /// The node finder is used to create new nodes.
    node_finder: NodeFinder,

    // /// The panning of the graph viewport.
    // pub pan_zoom: PanZoom,
    ///
    mouse_on_node: Option<MousePosOnNode>,

    graph_ref: NodeRef,

    _mouse_up_event: Option<EventListener>,

    _user_state: PhantomData<fn() -> UserState>,
    _template: PhantomData<fn() -> NodeTemplate>,
}
#[derive(Debug, Clone)]
pub enum GraphMessage<NodeTemplate> {
    SelectNode {
        id: NodeId,
        shift_key: bool,
    },

    DragStart {
        data: MousePosOnNode,
        shift_key: bool,
    },
    Dragging(Vec2),
    DragEnd,

    // NodeFinder Event
    OpenNodeFinder(Vec2),
    CreateNode(NodeTemplate),

    BackgroundClick,
    Rendered(NodeRef),

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
{
    type Message = GraphMessage<NodeTemplate>;
    type Properties = BasicGraphEditorProps<UserState>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            graph: Default::default(),
            selected_nodes: Default::default(),
            node_positions: Default::default(),
            node_finder: Default::default(),
            mouse_on_node: Default::default(),
            graph_ref: Default::default(),
            _mouse_up_event: Default::default(),
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
            GraphMessage::DragStart { data, shift_key } => {
                let document = window().document().unwrap();
                let on_mouse_move = ctx.link().callback(|()| GraphMessage::DragEnd);
                self._mouse_up_event = Some(EventListener::new(&document, "mouseup", move |_| {
                    on_mouse_move.emit(())
                }));

                if !shift_key {
                    self.selected_nodes.clear();
                }
                self.selected_nodes.insert(data.id);
                self.mouse_on_node = Some(data);
                false
            }
            GraphMessage::Dragging(pos) => {
                if let Some(MousePosOnNode { id, gap }) = self.mouse_on_node {
                    let pos = pos - gap;
                    let selected_pos = &mut self.node_positions[id];
                    let drag_delta = pos - *selected_pos;
                    *selected_pos = pos;
                    for id_ in &self.selected_nodes {
                        if &id != id_ {
                            self.node_positions[*id_] += drag_delta;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            GraphMessage::DragEnd => {
                self._mouse_up_event = None;
                self.mouse_on_node = None;
                false
            }
            GraphMessage::CreateNode(template) => {
                let new_node = self.graph.add_node(
                    template.node_graph_label(user_state),
                    template.user_data(user_state),
                    |graph, node_id| template.build_node(graph, user_state, node_id),
                );
                self.node_positions.insert(new_node, self.node_finder.pos);
                self.selected_nodes.insert(new_node);
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
            GraphMessage::Rendered(node_ref) => {
                self.graph_ref = node_ref;
                false
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
                NodeEvent::DragStart { gap, shift_key } => DragStart {
                    data: MousePosOnNode { id, gap },
                    shift_key,
                },
                NodeEvent::Port(_) => None,
            });
            html! {<Node<NodeData, DataType, ValueType>
                key={id.to_string()}
                data={self.graph[id].clone()}
                input_params={self.graph.inputs.clone()}
                output_params={self.graph.outputs.clone()}
                pos={self.node_positions[id]}
                is_selected={self.selected_nodes.contains(&id)}
                onevent={node_event}
            />}
        });

        let background_event = ctx.link().callback(|e: BackgroundEvent| match e {
            BackgroundEvent::ContextMenu(pos) => OpenNodeFinder(pos),
            BackgroundEvent::Click(_) => BackgroundClick,
            BackgroundEvent::Move(pos) => Dragging(pos),
            BackgroundEvent::MouseUp(_) => DragEnd,
            BackgroundEvent::Rendered(node_ref) => Rendered(node_ref),
        });

        html! {
            <GraphArea
                onevent={background_event}
            >
            {for nodes}
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
