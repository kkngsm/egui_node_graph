use glam::Vec2;
use yew::{function_component, html, Html, Properties};

/// Properties of [`SelectBox`]
#[derive(Properties, PartialEq)]
pub struct SelectBoxProps {
    pub start: Vec2,
    pub end: Vec2,
}

/// SelectBox component
///
/// The following are the HTML attributes of this component.
/// The minimum style that does not interfere with operation is set.
/// ```text
/// class: "select-box"
/// style:{
///     position:absolute;
///     left:{}px; top:{}px;
///     width:{}px; height:{}px;
/// }
/// ```
#[function_component(SelectBox)]
pub fn select_box(SelectBoxProps { start, end }: &SelectBoxProps) -> Html {
    let min = start.min(*end);
    let max = start.max(*end);
    let wh = max - min;
    let box_class = format!(
        "position:absolute;
left:{}px; top:{}px;
width:{}px; height:{}px;
",
        min.x, min.y, wh.x, wh.y
    );
    html! {
        <div class={"select-box"} style={box_class} />
    }
}
