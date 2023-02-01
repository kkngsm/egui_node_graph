use stylist::yew::styled_component;
use yew::prelude::*;
#[derive(Properties, PartialEq)]
pub struct GraphProps {
    #[prop_or_default]
    pub children: Children,
}
#[styled_component(GraphArea)]
pub fn graph_area(GraphProps { children }: &GraphProps) -> Html {
    let graph_area = css!(
        r#"
clip-path:inset(100%);
"#
    );
    html! {
        <div class={classes![graph_area,"graph-area"]}>
            { for children.iter() }
        </div>
    }
}
