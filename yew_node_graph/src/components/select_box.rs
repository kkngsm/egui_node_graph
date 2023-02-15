use glam::Vec2;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct SelectBoxProps {
    pub start: Vec2,
    pub end: Vec2,
}
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
        <div class={"select_box"} style={box_class} />
    }
}
