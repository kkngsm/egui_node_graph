use std::cell::RefCell;
use std::collections::HashSet;

use std::marker::PhantomData;

use std::rc::Rc;

use crate::state::{
    DragState, Graph, NodeDataTrait, NodeFinder, NodeId, NodeTemplateIter, NodeTemplateTrait,
    PortRefs, WidgetValueTrait,
};

use gloo::events::EventListener;
use gloo::utils::window;
use slotmap::SecondaryMap;
use wasm_bindgen::UnwrapThrowExt;
use yew::{Callback, NodeRef};

/// Basic GraphEditor components
/// The following limitations apply
/// - NodeFinder is the default
/// - UserState must implement PartialEq
/// If you want a broader implementation, you may want to define your own components
pub struct BasicGraphEditorState<NodeData, DataType, ValueType, NodeTemplate> {
    pub graph: Rc<RefCell<Graph<NodeData, DataType, ValueType>>>,
    //TODO
    // /// Nodes are drawn in this order. Draw order is important because nodes
    // /// that are drawn last are on top.
    // pub node_order: Vec<NodeId>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    pub selected_nodes: HashSet<NodeId>,

    /// The position of each node.
    pub node_positions: SecondaryMap<NodeId, crate::Vec2>,

    pub port_refs: PortRefs,

    pub node_finder: NodeFinder,

    // /// The panning of the graph viewport.
    // pub pan_zoom: PanZoom,
    ///
    pub graph_ref: NodeRef,

    pub drag_event: Option<DragState>,
    pub _drag_event_listener: Option<[EventListener; 2]>,

    _template: PhantomData<fn() -> NodeTemplate>,
}

impl<NodeData, DataType, ValueType, NodeTemplate> Default
    for BasicGraphEditorState<NodeData, DataType, ValueType, NodeTemplate>
{
    fn default() -> Self {
        Self {
            graph: Default::default(),
            selected_nodes: Default::default(),
            node_positions: Default::default(),
            port_refs: Default::default(),
            node_finder: Default::default(),
            graph_ref: Default::default(),
            drag_event: Default::default(),
            _drag_event_listener: Default::default(),
            _template: PhantomData,
        }
    }
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
    BasicGraphEditorState<NodeData, DataType, ValueType, NodeTemplate>
where
    NodeData: NodeDataTrait<
        DataType = DataType,
        ValueType = ValueType,
        UserState = UserState,
        Response = UserResponse,
    >,
    NodeTemplate: NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + NodeTemplateIter<Item = NodeTemplate>
        + 'static,
    ValueType:
        WidgetValueTrait<UserState = UserState, NodeData = NodeData, Response = UserResponse>,
    UserResponse: 'static,
{
    pub fn set_drag_event<Message>(
        &mut self,
        onmouseup: Callback<web_sys::Event>,
        onmousemove: Callback<web_sys::Event>,
    ) {
        let document = window().document().unwrap();

        self._drag_event_listener = Some([
            EventListener::new(&document, "mouseup", move |e| onmouseup.emit(e.to_owned())),
            EventListener::new(
                &self.graph_ref.cast::<web_sys::Element>().unwrap_throw(),
                "mousemove",
                move |e| onmousemove.emit(e.to_owned()),
            ),
        ]);
    }
}
