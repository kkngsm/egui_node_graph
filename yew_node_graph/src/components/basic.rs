use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::components::*;
use crate::state::{GraphEditorState, MousePosOnNode, NodeId, NodeTemplateIter, NodeTemplateTrait};
use crate::Vec2;
use gloo::events::EventListener;
use gloo::utils::window;
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
    state: GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, UserState>,
    drag_state: Option<MousePosOnNode>,
    _mouse_up_event: Option<EventListener>,
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
{
    type Message = GraphMessage<NodeTemplate>;
    type Properties = BasicGraphEditorProps<UserState>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: Default::default(),
            drag_state: Default::default(),
            _mouse_up_event: Default::default(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", &msg);
        let BasicGraphEditorProps { user_state } = ctx.props();
        let user_state = &mut *user_state.borrow_mut();
        match msg {
            GraphMessage::SelectNode { id, shift_key } => {
                if !shift_key {
                    self.state.selected_nodes.clear();
                }
                self.state.selected_nodes.insert(id);
                true
            }
            GraphMessage::DragStart { data, shift_key } => {
                let document = window().document().unwrap();
                let on_mouse_move = ctx.link().callback(|()| GraphMessage::DragEnd);
                self._mouse_up_event = Some(EventListener::new(&document, "mouseup", move |_| {
                    on_mouse_move.emit(())
                }));

                if !shift_key {
                    self.state.selected_nodes.clear();
                }
                self.state.selected_nodes.insert(data.id);
                self.drag_state = Some(data);
                false
            }
            GraphMessage::Dragging(pos) => {
                if let Some(MousePosOnNode { id, gap }) = self.drag_state {
                    let pos = pos - gap;
                    let selected_pos = &mut self.state.node_positions[id];
                    let drag_delta = pos - *selected_pos;
                    *selected_pos = pos;
                    for id_ in &self.state.selected_nodes {
                        if &id != id_ {
                            self.state.node_positions[*id_] += drag_delta;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            GraphMessage::DragEnd => {
                self._mouse_up_event = None;
                self.drag_state = None;
                false
            }
            GraphMessage::CreateNode(template) => {
                let new_node = self.state.graph.add_node(
                    template.node_graph_label(user_state),
                    template.user_data(user_state),
                    |graph, node_id| template.build_node(graph, user_state, node_id),
                );
                self.state
                    .node_positions
                    .insert(new_node, self.state.node_finder.pos);
                self.state.selected_nodes.insert(new_node);
                true
            }
            GraphMessage::OpenNodeFinder(pos) => {
                self.state.node_finder.is_showing = true;
                self.state.node_finder.pos = pos;
                true
            }
            GraphMessage::BackgroundClick => {
                let mut changed = false;
                let is_showing = &mut self.state.node_finder.is_showing;
                changed |= if *is_showing {
                    *is_showing = false;
                    true
                } else {
                    false
                };

                changed |= if self.state.selected_nodes.is_empty() {
                    false
                } else {
                    self.state.selected_nodes.clear();
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
        let nodes = self.state.graph.nodes.iter().map(|(id, node)| {
            let node_event = ctx.link().callback(move |e| match e {
                NodeEvent::Select { shift_key } => SelectNode { id, shift_key },
                NodeEvent::DragStart { gap, shift_key } => DragStart {
                    data: MousePosOnNode { id, gap },
                    shift_key,
                },
            });
            html! {<Node
                key={format!("{id:?}")}
                title={node.label.clone()}
                pos={self.state.node_positions[id]}
                is_selected={self.state.selected_nodes.contains(&id)}
                onevent={node_event}
            />}
        });

        let background_event = ctx.link().callback(|e: BackgroundEvent| match e {
            BackgroundEvent::ContextMenu(pos) => OpenNodeFinder(pos),
            BackgroundEvent::Click(_) => BackgroundClick,
            BackgroundEvent::Move(pos) => Dragging(pos),
            BackgroundEvent::MouseUp(_) => DragEnd,
        });

        html! {
            <GraphArea
                onevent={background_event}
            >
            {for nodes}
            <BasicNodeFinder<NodeTemplate, UserState>
                is_showing={self.state.node_finder.is_showing}
                pos={self.state.node_finder.pos}
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
