use crate::components::*;
use crate::state::GraphEditorState;
use crate::{vec2, Vec2};
use yew::prelude::*;

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

impl<NodeData, DataType, ValueType, NodeTemplate, UserState> Component
    for BasicGraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
{
    type Message = GraphMessage;
    type Properties = ();
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
            <ContextMenu is_showing={self.state.node_finder.is_showing} pos={self.state.node_finder.pos}></ContextMenu>
            </>
        }
    }
}
