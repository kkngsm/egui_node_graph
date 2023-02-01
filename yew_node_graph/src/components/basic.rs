use std::rc::Rc;

use crate::components::*;
use crate::state::{GraphEditorState, NodeTemplateIter, NodeTemplateTrait};
use crate::{vec2, Vec2};
use yew::prelude::*;
/// Basic GraphEditor components
/// The following limitations apply
/// - NodeFinder is the default
/// - UserState must implement PartialEq
/// If you want a broader implementation, you may want to define your own components
pub struct BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
where
    NodeData: 'static,
    DataType: 'static,
    ValueType: 'static,
    NodeTemplate: 'static,
    UserState: 'static,
{
    state: GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, UserState>,
}
#[derive(Debug, Clone)]
pub enum GraphMessage {
    NodeEvent(NodeEvent),
    OpenNodeFinder(Vec2),
    CloseNodeFinder,
}

/// Props for [`BasicGraphEditor`]
#[derive(Properties, PartialEq)]
pub struct BasicGraphEditorProps<UserState: PartialEq> {
    pub user_state: Rc<UserState>,
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
        > + NodeTemplateIter<Item = NodeTemplate>,
{
    type Message = GraphMessage;
    type Properties = BasicGraphEditorProps<UserState>;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: Default::default(),
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            GraphMessage::NodeEvent(_) => (),
            GraphMessage::OpenNodeFinder(pos) => {
                self.state.node_finder.is_showing = true;
                self.state.node_finder.pos = pos;
            }
            GraphMessage::CloseNodeFinder => self.state.node_finder.is_showing = false,
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        use GraphMessage::*;
        let BasicGraphEditorProps { user_state } = ctx.props();
        let nodes = self.state.graph.nodes.iter().map(|(id, node)| html!{<Node title={node.label.clone()} pos={self.state.node_positions[id]}/>});

        let background_event = ctx.link().callback(|e: BackgroundEvent| match e {
            BackgroundEvent::ContextMenu(e) => {
                let pos = vec2(e.client_x() as f32, e.client_y() as f32);
                OpenNodeFinder(pos)
            }
            BackgroundEvent::Click(_) => CloseNodeFinder,
        });
        html! {
            <>
            <GraphArea
                onevent={background_event}
            >
                {for nodes}
            </GraphArea>
            <BasicNodeFinder<NodeTemplate, UserState>
                is_showing={self.state.node_finder.is_showing}
                pos={self.state.node_finder.pos}
                user_state={user_state.clone()}
            />
            </>
        }
    }
}

#[derive(PartialEq, Properties)]
pub struct BasicNodeFinderProps<UserState>
where
    UserState: PartialEq,
{
    pub is_showing: bool,
    pub pos: Vec2,
    pub user_state: Rc<UserState>,
}

#[function_component(BasicNodeFinder)]
pub fn basic_finder<NodeTemplate, UserState>(
    BasicNodeFinderProps {
        is_showing,
        pos,
        user_state,
    }: &BasicNodeFinderProps<UserState>,
) -> Html
where
    NodeTemplate: NodeTemplateTrait<UserState = UserState> + NodeTemplateIter<Item = NodeTemplate>,
    UserState: PartialEq,
{
    let buttons = NodeTemplate::all_kinds().into_iter().map(|t| {
        html! {
            <li><button>{t.node_finder_label(&user_state)}</button></li>
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
