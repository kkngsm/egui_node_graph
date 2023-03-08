use std::collections::HashSet;

use std::marker::PhantomData;

use std::rc::Rc;

use crate::{
    state::{
        ConnectTo, DragState, Graph, NodeDataTrait, NodeFinder, NodeId, NodeTemplateIter,
        NodeTemplateTrait, PortRefs, WidgetValueTrait,
    },
    utils::{get_center, get_offset},
};

use glam::Vec2;
use slotmap::SecondaryMap;

use yew::NodeRef;

use super::{AnyParameterId, ConnectionInProgress, InputId, Node, OutputId, PanZoom};

/// Basic GraphEditor components
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct GraphEditorState<NodeData, DataType, ValueType, NodeTemplate> {
    pub graph: Rc<Graph<NodeData, DataType, ValueType>>,
    //TODO
    // /// Nodes are drawn in this order. Draw order is important because nodes
    // /// that are drawn last are on top.
    // pub node_order: Vec<NodeId>,
    /// The currently selected node. Some interface actions depend on the
    /// currently selected node.
    pub selected_nodes: Rc<HashSet<NodeId>>,

    /// The position of each node.
    pub node_positions: Rc<SecondaryMap<NodeId, crate::Vec2>>,

    pub port_refs: PortRefs,

    pub node_finder: NodeFinder,

    /// The panning of the graph viewport.
    pub pan_zoom: PanZoom,
    ///
    pub graph_ref: NodeRef,

    pub drag_state: Option<DragState>,

    _phantom: PhantomData<fn() -> NodeTemplate>,
}

impl<NodeData, DataType, ValueType, NodeTemplate> Default
    for GraphEditorState<NodeData, DataType, ValueType, NodeTemplate>
{
    fn default() -> Self {
        Self {
            graph: Default::default(),
            selected_nodes: Default::default(),
            node_positions: Default::default(),
            port_refs: Default::default(),
            node_finder: Default::default(),
            pan_zoom: Default::default(),
            graph_ref: Default::default(),
            drag_state: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<NodeData, DataType, ValueType, NodeTemplate, UserState, UserResponse>
    GraphEditorState<NodeData, DataType, ValueType, NodeTemplate>
where
    NodeData: NodeDataTrait<
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
            Response = UserResponse,
        > + Clone,
    DataType: Clone + PartialEq,
    ValueType: WidgetValueTrait<UserState = UserState, NodeData = NodeData, Response = UserResponse>
        + Clone,
    NodeTemplate: NodeTemplateTrait<
            NodeData = NodeData,
            DataType = DataType,
            ValueType = ValueType,
            UserState = UserState,
        > + NodeTemplateIter<Item = NodeTemplate>,
    UserResponse: 'static,
{
    #[cfg_attr(doc, aquamarine::aquamarine)]
    /// ```mermaid
    /// graph LR
    /// Node --> select?{is selected?}
    /// select? -- yes --> selected_shift?{is shift key<br>pressed?}
    /// select? -- no --> unselected_shift?{is shift key<br>pressed?}

    /// selected_shift? -- yes --> r[Remove from Selection]
    /// selected_shift? -- no --> Nothing to do

    /// unselected_shift? -- yes --> i[Inclusive Select]
    /// unselected_shift? -- no --> e[Exclusive Select]
    /// ```
    pub fn selection(&mut self, id: NodeId, is_shift_key_pressed: bool) {
        if is_shift_key_pressed {
            if self.selected_nodes.contains(&id) {
                self.unselect(id);
            } else {
                self.inclusive_select(id);
            }
        } else {
            self.exclusive_select(id);
        }
    }
    pub fn exclusive_select(&mut self, node_id: NodeId) {
        let selected_nodes = Rc::make_mut(&mut self.selected_nodes);
        selected_nodes.clear();
        selected_nodes.insert(node_id);
    }
    pub fn inclusive_select(&mut self, node_id: NodeId) {
        let selected_nodes = Rc::make_mut(&mut self.selected_nodes);
        selected_nodes.insert(node_id);
    }
    pub fn unselect(&mut self, node_id: NodeId) {
        let selected_nodes = Rc::make_mut(&mut self.selected_nodes);
        selected_nodes.remove(&node_id);
    }

    pub fn create_node(&mut self, template: NodeTemplate, user_state: &UserState) -> NodeId {
        let new_node = Rc::make_mut(&mut self.graph).add_node(
            template.node_graph_label(user_state),
            template.user_data(user_state),
            |graph, node_id| template.build_node(graph, user_state, node_id),
        );
        let pos = self.pan_zoom.screen2logical(self.node_finder.pos);

        let selected_nodes = Rc::make_mut(&mut self.selected_nodes);
        let node_positions = Rc::make_mut(&mut self.node_positions);
        node_positions.insert(new_node, pos);
        selected_nodes.insert(new_node);

        let node = &self.graph[new_node];
        let mut port_refs = self.port_refs.borrow_mut();
        for input in node.input_ids() {
            port_refs.input.insert(input, Default::default());
        }
        for output in node.output_ids() {
            port_refs.output.insert(output, Default::default());
        }
        new_node
    }

    pub fn delete_node(
        &mut self,
        node_id: NodeId,
    ) -> (Rc<Node<NodeData>>, Vec<(InputId, OutputId)>) {
        let selected_nodes = Rc::make_mut(&mut self.selected_nodes);
        selected_nodes.remove(&node_id);
        let (node, disconnected) = Rc::make_mut(&mut self.graph).remove_node(node_id);
        let mut port_refs = self.port_refs.borrow_mut();
        for input in node.input_ids() {
            port_refs.input.remove(input);
        }
        for output in node.output_ids() {
            port_refs.output.remove(output);
        }
        (node, disconnected)
    }

    pub fn open_node_finder(&mut self, pos: Vec2) {
        self.node_finder.is_showing = true;
        self.node_finder.pos = pos;
    }

    pub fn start_moving_node(&mut self, id: NodeId, shift: Vec2) {
        if self.selected_nodes.contains(&id) {
            self.drag_state = Some(DragState::MoveNode { id, shift });
        }
    }
    pub fn move_node(&mut self, pos: Vec2) -> Option<Vec2> {
        if let Some(DragState::MoveNode { id, shift, .. }) = self.drag_state.as_mut() {
            let pos = pos - *shift;
            let selected_pos = self.node_positions[*id];
            let drag_delta = self.pan_zoom.screen2logical(pos) - selected_pos;
            let node_positions = Rc::make_mut(&mut self.node_positions);
            for id in self.selected_nodes.iter().copied() {
                node_positions[id] += drag_delta;
            }
            Some(drag_delta)
        } else {
            None
        }
    }
    pub fn end_moving_node(&mut self) {
        self.drag_state.take();
    }

    pub fn start_connection(&mut self, id: AnyParameterId) -> Option<(OutputId, InputId)> {
        let pos = self.port_refs.borrow().get(id).and_then(get_center);
        let offset = get_offset(&self.graph_ref);
        let pos = pos.zip(offset).map(|(p, o)| p - o).unwrap_or_default();

        match id {
            AnyParameterId::Input(input) => {
                if let Some(output) = Rc::make_mut(&mut self.graph).connections.remove(input) {
                    self.drag_state = Some(DragState::ConnectPort((output, pos).into()));
                    Some((output, input))
                } else {
                    self.drag_state = Some(DragState::ConnectPort((input, pos).into()));
                    None
                }
            }
            AnyParameterId::Output(output) => {
                self.drag_state = Some(DragState::ConnectPort((output, pos).into()));
                None
            }
        }
    }
    pub fn move_connection(&mut self, pos: Vec2) {
        if let Some(DragState::ConnectPort(c)) = self.drag_state.as_mut() {
            if let ConnectionInProgress::FromInput {
                dest: ConnectTo::Pos(p),
                ..
            }
            | ConnectionInProgress::FromOutput {
                dest: ConnectTo::Pos(p),
                ..
            } = c
            {
                *p = pos;
            }
        }
    }
    pub fn end_connection(&mut self) -> Option<(OutputId, InputId)> {
        if let Some(DragState::ConnectPort(c)) = self.drag_state.take() {
            // Connect to Port
            if let Some((&output, &input)) = c.pair() {
                if self.graph.param_typ_eq(output, input) {
                    Rc::make_mut(&mut self.graph)
                        .connections
                        .insert(input, output);
                    return Some((output, input));
                }
            }
        }
        None
    }

    pub fn start_select_box(&mut self, start: Vec2) {
        self.drag_state = Some(DragState::SelectBox { start, end: start });
    }
    pub fn scale_select_box(&mut self, end: Vec2) {
        if let Some(DragState::SelectBox { end: e, .. }) = self.drag_state.as_mut() {
            *e = end;
        }
    }
    pub fn end_select_box(&mut self) {
        if let Some(DragState::SelectBox { end, start }) = self.drag_state.take() {
            let min = self.pan_zoom.screen2logical(start.min(end));
            let max = self.pan_zoom.screen2logical(start.max(end));

            for id in self
                .node_positions
                .iter()
                .flat_map(|(id, pos)| (min.cmplt(*pos).all() && pos.cmplt(max).all()).then_some(id))
            {
                Rc::make_mut(&mut self.selected_nodes).insert(id);
            }
        }
    }

    pub fn connection_in_progress(&self) -> Option<(Vec2, Vec2, DataType)> {
        fn src_dest_pos<Src: slotmap::Key, Dest: slotmap::Key>(
            src: Src,
            src_map: &SecondaryMap<Src, NodeRef>,
            dest: &ConnectTo<Dest>,
            dest_map: &SecondaryMap<Dest, NodeRef>,
            offset: Vec2,
        ) -> (Vec2, Vec2) {
            (
                src_map
                    .get(src)
                    .and_then(get_center)
                    .map(|p| p - offset)
                    .unwrap_or_default(),
                dest.map_pos(|id| {
                    dest_map
                        .get(*id)
                        .and_then(get_center)
                        .map(|p| p - offset)
                        .unwrap_or_default()
                }),
            )
        }

        if let (Some(DragState::ConnectPort(c)), Some(offset)) =
            (&self.drag_state, get_offset(&self.graph_ref))
        {
            let port_refs = self.port_refs.borrow();
            let r = match c {
                ConnectionInProgress::FromInput { src, dest } => {
                    let (src_pos, dest_pos) =
                        src_dest_pos(*src, &port_refs.input, dest, &port_refs.output, offset);
                    (dest_pos, src_pos, self.graph.inputs[*src].typ.clone())
                }
                ConnectionInProgress::FromOutput { src, dest } => {
                    let (src_pos, dest_pos) =
                        src_dest_pos(*src, &port_refs.output, dest, &port_refs.input, offset);
                    (src_pos, dest_pos, self.graph.outputs[*src].typ.clone())
                }
            };
            Some(r)
        } else {
            None
        }
    }

    pub fn select_box(&self) -> Option<(Vec2, Vec2)> {
        if let Some(DragState::SelectBox { start, end }) = &self.drag_state {
            Some((*start, *end))
        } else {
            None
        }
    }
}
