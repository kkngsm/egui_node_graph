pub use glam::{vec2, Vec2};
use model::GraphEditorState;
use yew::prelude::*;
mod hooks;
pub mod model;
pub mod view;
use view::*;

pub struct GraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
where
    NodeData: 'static,
    DataType: 'static,
    ValueType: 'static,
    NodeTemplate: 'static,
    UserState: 'static,
{
    state: GraphEditorState<NodeData, DataType, ValueType, NodeTemplate, UserState>,
}
#[derive(Debug, Clone, Copy)]
pub enum GraphMessage {
    NodeEvent(NodeEvent),
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState> Component
    for GraphEditor<NodeData, DataType, ValueType, NodeTemplate, UserState>
{
    type Message = GraphMessage;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: Default::default(),
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", msg);
        // match msg {
        //     GraphMessage::NodeEvent(node) => match node {
        // NodeEvent::Drag(drag) => match drag {
        //     hooks::DragEvent::Start => false,
        //     hooks::DragEvent::Move(pos, _) => {
        //         self.pos = pos;
        //         true
        //     }
        //     hooks::DragEvent::End => false,
        // },
        //     },
        // }
        true
    }
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let nodes = self.state.graph.nodes.iter().map(|(id, node)| html!{<Node title={node.label.clone()} pos={self.state.node_positions[id]}/>});
        html! {
            <GraphArea
                onevent={|e| log::debug!("{:?}", e)}
            >
                {for nodes}
            </GraphArea>
        }
    }
}
