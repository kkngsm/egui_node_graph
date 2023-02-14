#[derive(Default, Copy, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct PanZoom {
    pub pan: crate::Vec2,
    pub zoom: f32,
}

impl PanZoom {
    pub fn adjust_zoom(
        &mut self,
        zoom_delta: f32,
        point: crate::Vec2,
        zoom_min: f32,
        zoom_max: f32,
    ) {
        let zoom_clamped = (self.zoom + zoom_delta).clamp(zoom_min, zoom_max);
        let zoom_delta = zoom_clamped - self.zoom;

        self.zoom += zoom_delta;
        self.pan += point * zoom_delta;
    }
}

/// NodeFinder Status
/// this is used to create new nodes.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeFinder {
    pub pos: crate::Vec2,
    pub is_showing: bool,
}

use std::{cell::RefCell, ops::Index, rc::Rc};

use glam::Vec2;
use slotmap::SecondaryMap;

use super::{AnyParameterId, InputId, NodeId, OutputId};

// Information needed when dragging or selecting a node
#[derive(Debug, Clone)]
pub struct MousePosOnNode {
    /// Id of mouse-on node
    pub id: NodeId,
    /// Position from top left of node
    pub gap: Vec2,
}

#[derive(Debug, Clone, Default)]
pub struct PortsData<T> {
    pub input: SecondaryMap<InputId, T>,
    pub output: SecondaryMap<OutputId, T>,
}

impl<T> PortsData<T> {
    pub fn insert(&mut self, key: AnyParameterId, value: T) -> Option<T> {
        match key {
            AnyParameterId::Input(id) => self.input.insert(id, value),
            AnyParameterId::Output(id) => self.output.insert(id, value),
        }
    }
    pub fn remove(&mut self, key: AnyParameterId) -> Option<T> {
        match key {
            AnyParameterId::Input(id) => self.input.remove(id),
            AnyParameterId::Output(id) => self.output.remove(id),
        }
    }
    pub fn get(&self, key: AnyParameterId) -> Option<&T> {
        match key {
            AnyParameterId::Input(id) => self.input.get(id),
            AnyParameterId::Output(id) => self.output.get(id),
        }
    }
    pub fn get_mut(&mut self, key: AnyParameterId) -> Option<&mut T> {
        match key {
            AnyParameterId::Input(id) => self.input.get_mut(id),
            AnyParameterId::Output(id) => self.output.get_mut(id),
        }
    }
}

impl<T> Index<InputId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: InputId) -> &Self::Output {
        &self.input[index]
    }
}
impl<T> Index<OutputId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: OutputId) -> &Self::Output {
        &self.output[index]
    }
}

impl<T> Index<AnyParameterId> for PortsData<T> {
    type Output = T;
    fn index(&self, index: AnyParameterId) -> &Self::Output {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "{} index error for {:?}. Has the value been deleted?",
                stringify!(AnyParameterId),
                index
            )
        })
    }
}

/// [`yew::NodeRef`] of each port.
pub type PortRefs = Rc<RefCell<PortsData<yew::NodeRef>>>;

/// this have Port or free (mouse) where the connection is going.
/// This is mainly used in [`ConnectionInProgress`]
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectTo<Id> {
    Id(Id),
    Pos(Vec2),
}
impl<Id> ConnectTo<Id> {
    /// if [`ConnectTo::Pos`] return inner
    /// if [`ConnectTo::Id`] execute the function of the argument and return the return value
    ///
    /// #Example
    /// ```
    /// use yew_node_graph::vec2;
    /// use yew_node_graph::state::ConnectTo;
    ///
    /// let map = vec![vec2(11.0, 15.0)];
    /// let connect_to_id = ConnectTo::Id(0usize);
    ///
    /// let f = |id: &usize| map[*id];
    /// assert_eq!(connect_to_id.map_pos(f), vec2(11.0, 15.0));
    ///
    /// let connect_to_pos= ConnectTo::Pos(vec2(32.0, 24.0));
    /// assert_eq!(connect_to_pos.map_pos(f), vec2(32.0, 24.0))
    /// ```
    pub fn map_pos(&self, f: impl Fn(&Id) -> Vec2) -> Vec2 {
        match self {
            ConnectTo::Id(id) => f(id),
            ConnectTo::Pos(pos) => *pos,
        }
    }
}
impl From<InputId> for ConnectTo<InputId> {
    fn from(value: InputId) -> Self {
        Self::Id(value)
    }
}
impl From<OutputId> for ConnectTo<OutputId> {
    fn from(value: OutputId) -> Self {
        Self::Id(value)
    }
}
impl<Id> From<Vec2> for ConnectTo<Id> {
    fn from(value: Vec2) -> Self {
        Self::Pos(value)
    }
}

/// An ongoing connection interaction: The mouse has dragged away from a
/// port and the user is holding the click
#[derive(Debug, Default, Clone, PartialEq)]
pub enum ConnectionInProgress {
    FromInput {
        src: InputId,
        dest: ConnectTo<OutputId>,
    },
    FromOutput {
        src: OutputId,
        dest: ConnectTo<InputId>,
    },
    #[default]
    None,
}

impl ConnectionInProgress {
    /// Set the destination to Port
    /// #Example
    /// ```
    /// use yew_node_graph::{
    ///     state::{ConnectTo, ConnectionInProgress, InputId, OutputId},
    ///     vec2,
    /// };
    /// fn example(input: InputId, output: OutputId, another_input: InputId) {
    ///     let mut connection_in_progress = ConnectionInProgress::FromInput {
    ///         src: input,
    ///         dest: ConnectTo::Pos(vec2(1.0, 2.0)),
    ///     };
    ///     connection_in_progress.to_id(output.into());
    ///     assert_eq!(
    ///         connection_in_progress,
    ///         ConnectionInProgress::FromInput {
    ///             src: input,
    ///             dest: ConnectTo::Id(output),
    ///         }
    ///     );
    ///
    ///     // Do nothing if the following
    ///
    ///     // Inputs cannot be connected to inputs.
    ///     connection_in_progress.to_id(another_input.into());
    ///
    ///     let mut connection_none = ConnectionInProgress::None;
    ///     connection_none.to_id(output.into());
    /// }
    /// ```
    pub fn to_id(&mut self, id: AnyParameterId) {
        match (self, id) {
            (ConnectionInProgress::FromInput { src: _, dest }, AnyParameterId::Output(id)) => {
                *dest = id.into()
            }
            (ConnectionInProgress::FromOutput { src: _, dest }, AnyParameterId::Input(id)) => {
                *dest = id.into()
            }
            _ => (),
        }
    }
    /// Set the destination to Position
    /// #Example
    /// ```
    /// use yew_node_graph::{
    ///     state::{ConnectTo, ConnectionInProgress, InputId, OutputId},
    ///     vec2,
    /// };
    /// fn example(input: InputId, output: OutputId) {
    ///     let mut connection_in_progress = ConnectionInProgress::FromInput {
    ///         src: input,
    ///         dest: ConnectTo::Id(output),
    ///     };
    ///     connection_in_progress.to_pos(vec2(12.0, 34.0));
    ///     assert_eq!(
    ///         connection_in_progress,
    ///         ConnectionInProgress::FromInput {
    ///             src: input,
    ///             dest: ConnectTo::Pos(vec2(12.0, 34.0)),
    ///         }
    ///     );
    ///
    ///     // Do nothing if the following
    ///     let mut connection_none = ConnectionInProgress::None;
    ///     connection_none.to_pos(vec2(53.0, 65.0));
    /// }
    /// ```
    pub fn to_pos(&mut self, pos: Vec2) {
        match self {
            ConnectionInProgress::FromInput { src: _, dest } => *dest = pos.into(),
            ConnectionInProgress::FromOutput { src: _, dest } => *dest = pos.into(),
            ConnectionInProgress::None => (),
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl From<(AnyParameterId, Vec2)> for ConnectionInProgress {
    fn from((id, pos): (AnyParameterId, Vec2)) -> Self {
        match id {
            AnyParameterId::Input(id) => Self::FromInput {
                src: id,
                dest: pos.into(),
            },
            AnyParameterId::Output(id) => Self::FromOutput {
                src: id,
                dest: pos.into(),
            },
        }
    }
}

impl From<(OutputId, Vec2)> for ConnectionInProgress {
    fn from((id, pos): (OutputId, Vec2)) -> Self {
        Self::FromOutput {
            src: id,
            dest: pos.into(),
        }
    }
}

impl From<(InputId, Vec2)> for ConnectionInProgress {
    fn from((id, pos): (InputId, Vec2)) -> Self {
        Self::FromInput {
            src: id,
            dest: pos.into(),
        }
    }
}
