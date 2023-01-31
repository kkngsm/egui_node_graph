use yew_node_graph::Graph;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<Graph>::new().render();
}
