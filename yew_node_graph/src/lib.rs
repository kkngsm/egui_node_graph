use glam::Vec2;
use yew::prelude::*;
mod hooks;
mod view;
use view::*;

#[derive(Debug, Default)]
pub struct Graph {
    pos: Vec2,
}
#[derive(Debug, Clone, Copy)]
pub enum GraphMessage {
    NodeEvent(NodeEvent),
}

impl Component for Graph {
    type Message = GraphMessage;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        Default::default()
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", msg);
        match msg {
            GraphMessage::NodeEvent(node) => match node {
                NodeEvent::Drag(drag) => match drag {
                    hooks::DragEvent::Start => false,
                    hooks::DragEvent::Move(pos, _) => {
                        self.pos = pos;
                        true
                    }
                    hooks::DragEvent::End => false,
                },
            },
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <GraphArea>
                <Node title="String" pos={self.pos} onevent={ctx.link().callback(|event| Self::Message::NodeEvent(event))}/>
            </GraphArea>
        }
    }
}