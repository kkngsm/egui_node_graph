use std::fmt::Display;

use glam::{vec2, Vec2};
use stylist::yew::styled_component;
use yew::{html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct EdgeProps<DataType>
where
    DataType: Display + Clone + PartialEq + 'static,
{
    pub output: Vec2,
    pub input: Vec2,
    pub typ: DataType,
}
#[styled_component(Edge)]
pub fn edge<DataType>(EdgeProps { output, input, typ }: &EdgeProps<DataType>) -> Html
where
    DataType: Display + Clone + PartialEq + 'static,
{
    let output_dir = *output + vec2(100.0, 0.0);
    let input_dir = *input - vec2(100.0, 0.0);

    let svg_class = css! {
"position:absolute;
top:0px;
left:0px;
pointer-events: none;
"};
    html! {
            <svg xmlns={"http://www.w3.org/2000/svg"}
                class={svg_class}
                height={"100%"} width={"100%"}>
                <path class={"edge"}
                    data-type={typ.clone().to_string()}
                    d={format!(
                        "M {} {} C {} {}, {} {}, {} {}",
                        output.x, output.y,
                        output_dir.x,output_dir.y,
                        input_dir.x,input_dir.y,
                        input.x,input.y,
                    )}
                    fill={"none"}
                />
            </svg>
    }
}
