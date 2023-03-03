use glam::Vec2;
use web_sys::MouseEvent;
use yew::{function_component, html, Callback, Html, Properties};

use crate::{
    components::ContextMenu,
    state::{NodeTemplateIter, NodeTemplateTrait},
};

/// Properties of [`NodeFinder`]
#[derive(PartialEq, Properties)]
pub struct NodeFinderProps<NodeTemplate, UserState>
where
    NodeTemplate: PartialEq,
    UserState: PartialEq,
{
    pub is_showing: bool,
    pub pos: Vec2,
    pub user_state: UserState,
    pub onevent: Callback<NodeTemplate>,
}

#[function_component(NodeFinder)]
pub fn node_finder<NodeTemplate, UserState>(
    NodeFinderProps {
        is_showing,
        pos,
        user_state,
        onevent,
    }: &NodeFinderProps<NodeTemplate, UserState>,
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
            <li><button
                onclick={move |_| onevent.emit(t)}
                onmousedown={move |e:MouseEvent| e.stop_propagation()}
            >
                {t.node_finder_label(user_state)}
            </button></li>
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
