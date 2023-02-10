use std::{
    cell::{Ref, RefMut},
    rc::Rc,
};

use super::*;

impl<NodeData, DataType, ValueType> Graph<NodeData, DataType, ValueType> {
    pub fn new() -> Self {
        Self {
            nodes: Default::default(),
            inputs: Default::default(),
            outputs: Default::default(),
            connections: Default::default(),
        }
    }
    pub fn add_node(
        &mut self,
        label: String,
        user_data: NodeData,
        f: impl FnOnce(&mut Graph<NodeData, DataType, ValueType>, NodeId),
    ) -> NodeId {
        let node_id = self.nodes.insert_with_key(|node_id| {
            Rc::new(Node {
                id: node_id,
                label,
                // These get filled in later by the user function
                inputs: Vec::default(),
                outputs: Vec::default(),
                user_data,
            })
        });

        f(self, node_id);

        node_id
    }

    pub fn add_input_param(
        &mut self,
        node_id: NodeId,
        name: String,
        typ: DataType,
        value: ValueType,
        kind: InputParamKind,
        shown_inline: bool,
    ) -> InputId
    where
        NodeData: Clone,
    {
        let input_id = self.inputs.borrow_mut().insert_with_key(|input_id| {
            Rc::new(InputParam {
                id: input_id,
                typ,
                value,
                kind,
                node: node_id,
                shown_inline,
            })
        });
        Rc::make_mut(&mut self.nodes[node_id])
            .inputs
            .push((name, input_id));
        input_id
    }

    pub fn remove_input_param(&mut self, param: InputId)
    where
        NodeData: Clone,
    {
        let node_id = self.inputs.borrow()[param].node;

        Rc::make_mut(&mut self.nodes[node_id])
            .inputs
            .retain(|(_, id)| *id != param);
        self.inputs.borrow_mut().remove(param);
        self.connections_mut().retain(|i, _| i != param);
    }

    pub fn remove_output_param(&mut self, param: OutputId)
    where
        NodeData: Clone,
    {
        let node_id = self.outputs.borrow()[param].node;
        Rc::make_mut(&mut self.nodes[node_id])
            .outputs
            .retain(|(_, id)| *id != param);
        self.outputs.borrow_mut().remove(param);
        self.connections_mut().retain(|_, o| *o != param);
    }

    pub fn add_output_param(&mut self, node_id: NodeId, name: String, typ: DataType) -> OutputId
    where
        NodeData: Clone,
    {
        let output_id = self.outputs.borrow_mut().insert_with_key(|output_id| {
            Rc::new(OutputParam {
                id: output_id,
                node: node_id,
                typ,
            })
        });
        Rc::make_mut(&mut self.nodes[node_id])
            .outputs
            .push((name, output_id));
        output_id
    }

    /// Removes a node from the graph with given `node_id`. This also removes
    /// any incoming or outgoing connections from that node
    ///
    /// This function returns the list of connections that has been removed
    /// after deleting this node as input-output pairs. Note that one of the two
    /// ids in the pair (the one on `node_id`'s end) will be invalid after
    /// calling this function.
    pub fn remove_node(
        &mut self,
        node_id: NodeId,
    ) -> (Rc<Node<NodeData>>, Vec<(InputId, OutputId)>) {
        let mut disconnect_events = vec![];

        self.connections_mut().retain(|i, o| {
            if self.outputs.borrow()[*o].node == node_id || self.inputs.borrow()[i].node == node_id
            {
                disconnect_events.push((i, *o));
                false
            } else {
                true
            }
        });

        for input in self[node_id].input_ids() {
            self.inputs.borrow_mut().remove(input);
        }
        for output in self[node_id].output_ids() {
            self.outputs.borrow_mut().remove(output);
        }
        let removed_node = self.nodes.remove(node_id).expect("Node should exist");

        (removed_node, disconnect_events)
    }

    pub fn remove_connection(&mut self, input_id: InputId) -> Option<OutputId> {
        self.connections_mut().remove(input_id)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().map(|(id, _)| id)
    }

    pub fn add_connection(&mut self, output: OutputId, input: InputId) {
        self.connections_mut().insert(input, output);
    }

    pub fn connection(&self, input: InputId) -> Option<OutputId> {
        self.connections().get(input).copied()
    }

    pub fn any_param_type(&self, param: AnyParameterId) -> Result<DataType, EguiGraphError>
    where
        DataType: Clone,
    {
        match param {
            AnyParameterId::Input(input) => self.inputs.borrow().get(input).map(|x| x.typ.clone()),
            AnyParameterId::Output(output) => {
                self.outputs.borrow().get(output).map(|x| x.typ.clone())
            }
        }
        .ok_or(EguiGraphError::InvalidParameterId(param))
    }

    pub fn get_input(&self, input: InputId) -> Option<Rc<InputParam<DataType, ValueType>>> {
        self.inputs.borrow().get(input).cloned()
    }

    pub fn input(&self, input: InputId) -> Rc<InputParam<DataType, ValueType>> {
        self.inputs.borrow()[input].clone()
    }

    pub fn get_output(&self, output: OutputId) -> Option<Rc<OutputParam<DataType>>> {
        self.outputs.borrow().get(output).cloned()
    }

    pub fn output(&self, output: OutputId) -> Rc<OutputParam<DataType>> {
        self.outputs.borrow()[output].clone()
    }

    pub fn connections(&self) -> Ref<'_, SecondaryMap<InputId, OutputId>> {
        self.connections.borrow()
    }
    pub fn connections_mut(&self) -> RefMut<'_, SecondaryMap<InputId, OutputId>> {
        self.connections.borrow_mut()
    }

    pub fn param_typ_eq(&self, output: OutputId, input: InputId) -> bool
    where
        DataType: PartialEq,
    {
        self.outputs.borrow()[output].typ == self.inputs.borrow()[input].typ
    }
}

impl<NodeData, DataType, ValueType> Default for Graph<NodeData, DataType, ValueType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<NodeData> Node<NodeData> {
    pub fn inputs<'a, DataType, ValueType>(
        &'a self,
        graph: &'a Graph<NodeData, DataType, ValueType>,
    ) -> impl Iterator<Item = Rc<InputParam<DataType, ValueType>>> + 'a
    where
        NodeData: Clone,
        DataType: Clone,
        ValueType: Clone,
    {
        self.input_ids().map(|id| graph.inputs.borrow()[id].clone())
    }

    pub fn outputs<'a, DataType, ValueType>(
        &'a self,
        graph: &'a Graph<NodeData, DataType, ValueType>,
    ) -> impl Iterator<Item = Rc<OutputParam<DataType>>> + 'a
    where
        NodeData: Clone,
        DataType: Clone,
        ValueType: Clone,
    {
        self.output_ids()
            .map(|id| graph.outputs.borrow()[id].clone())
    }

    pub fn input_ids(&self) -> impl Iterator<Item = InputId> + '_ {
        self.inputs.iter().map(|(_name, id)| *id)
    }

    pub fn output_ids(&self) -> impl Iterator<Item = OutputId> + '_ {
        self.outputs.iter().map(|(_name, id)| *id)
    }

    pub fn get_input(&self, name: &str) -> Result<InputId, EguiGraphError> {
        self.inputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }

    pub fn get_output(&self, name: &str) -> Result<OutputId, EguiGraphError> {
        self.outputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }
}

impl InputParamKind {
    pub fn is_should_draw(&self) -> bool {
        match self {
            InputParamKind::ConnectionOnly => true,
            InputParamKind::ConstantOnly => false,
            InputParamKind::ConnectionOrConstant => true,
        }
    }
}
