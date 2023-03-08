use std::{cell::RefCell, ops::Index, rc::Rc};

use glam::Vec2;
use slotmap::SecondaryMap;

#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
/// A 2D affine transform, witch specializes only in translation and scaling.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PanZoom {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for PanZoom {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl PanZoom {
    /// Zoom around the specified coordinates.
    /// Objects at the specified coordinates do not move.
    ///```rust
    /// use yew_node_graph::{vec2, state::PanZoom};
    ///
    /// let p_a = vec2(24.0, 43.0);
    /// let p_b = vec2(8.0, 58.0);
    ///
    /// // zooming around p_a
    /// let pan_zoom = PanZoom::from_zoom_to_pos(3.0, p_a);
    ///
    /// assert_eq!(pan_zoom, PanZoom{
    ///     pan: vec2(-48.0, -86.0),
    ///     zoom: 3.0
    /// });
    ///
    /// // p_a does not move
    /// assert_eq!(pan_zoom.logical2screen(p_a), p_a);
    ///
    /// // p_b is zoomed around p_a
    /// assert_eq!(pan_zoom.logical2screen(p_b), vec2(-24.0, 88.0));
    /// ```
    pub fn from_zoom_to_pos(zoom: f32, pos: Vec2) -> Self {
        Self {
            zoom,
            pan: pos * (1.0 - zoom),
        }
    }
    /// Zoom around the specified coordinates.
    /// Objects at the specified coordinates do not move.
    ///
    /// This is the same as `pan_zoom = pan_zoom * PanZoom::from_zoom_to_pos(zoom, pos);`
    ///```rust
    /// use yew_node_graph::{vec2, state::PanZoom};
    ///
    /// let mut pan_zoom = PanZoom{
    ///     pan: vec2(35.0, 23.0),
    ///     zoom: 2.0
    /// };
    ///
    /// pan_zoom.zoom_to_pos(3.0, vec2(94.0, 38.0));
    /// assert_eq!(pan_zoom, PanZoom{
    ///     pan: vec2(-341.0, -129.0),
    ///     zoom: 6.0
    /// });
    ///
    /// let p = vec2(52.0, 8.0);
    /// let screen_p = pan_zoom.logical2screen(p);
    ///
    /// // If you want to specify a zoom factor, you can do this.
    /// let zoom_rate = 12.0 / pan_zoom.zoom;
    /// pan_zoom.zoom_to_pos(zoom_rate, p);
    /// assert_eq!(pan_zoom, PanZoom{
    ///     pan: vec2(-653.0, -177.0),
    ///     zoom: 12.0
    /// });
    /// // In this case, too, the object at the center of the zoom does not move.
    /// assert_eq!(pan_zoom.logical2screen(p), screen_p);
    /// ```
    pub fn zoom_to_pos(&mut self, zoom: f32, logical_pos: Vec2) {
        *self = *self * PanZoom::from_zoom_to_pos(zoom, logical_pos);
    }

    pub fn logical2screen(&self, pos: Vec2) -> Vec2 {
        self * pos
    }

    pub fn screen2logical(&self, pos: Vec2) -> Vec2 {
        self.inverse() * pos
    }

    pub fn pan_screen(&mut self, delta: Vec2) {
        self.pan += delta;
    }
    pub fn pan_logical(&mut self, delta: Vec2) {
        self.pan += delta * self.zoom
    }

    fn inverse(&self) -> Self {
        let iz = 1.0 / self.zoom;
        Self {
            pan: -self.pan * iz,
            zoom: iz,
        }
    }
}

macro_rules! impl_panzoom_ops {
    ($panzoom: ty) => {
        impl std::ops::Mul<Vec2> for $panzoom {
            type Output = Vec2;
            fn mul(self, rhs: Vec2) -> Self::Output {
                self.zoom * rhs + self.pan
            }
        }

        impl std::ops::Mul<PanZoom> for $panzoom {
            type Output = PanZoom;
            fn mul(self, rhs: PanZoom) -> Self::Output {
                PanZoom {
                    pan: self.pan + rhs.pan * self.zoom,
                    zoom: self.zoom * rhs.zoom,
                }
            }
        }
    };
}
impl_panzoom_ops!(PanZoom);
impl_panzoom_ops!(&PanZoom);
#[cfg(test)]
mod test {
    use glam::vec2;

    use super::PanZoom;

    #[test]
    fn mul_v2_test() {
        let pan_zoom = PanZoom {
            pan: vec2(43.0, 58.0),
            zoom: 3.0,
        };
        assert_eq!(pan_zoom * vec2(74.0, 34.0), vec2(265.0, 160.0))
    }

    #[cfg(test)]
    #[test]
    fn mul_self_test() {
        use super::PanZoom;

        let pan_zoom = PanZoom {
            pan: vec2(43.0, 58.0),
            zoom: 3.0,
        };
        let another = PanZoom {
            pan: vec2(25.0, 79.0),
            zoom: 4.0,
        };
        assert_eq!(
            pan_zoom * another,
            PanZoom {
                pan: vec2(118.0, 295.0),
                zoom: 12.0,
            }
        )
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

use super::{AnyParameterId, InputId, NodeId, OutputId};
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionInProgress {
    FromInput {
        src: InputId,
        dest: ConnectTo<OutputId>,
    },
    FromOutput {
        src: OutputId,
        dest: ConnectTo<InputId>,
    },
}

impl ConnectionInProgress {
    /// Change the destination to PortId from Position
    /// # Example
    /// ```rust
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
    ///     // Input cannot be connected to input.
    ///     connection_in_progress.to_id(another_input.into());
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
    /// Change the destination to Position from Port
    /// # Example
    /// ```rust
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
    /// }
    /// ```
    pub fn to_pos(&mut self, pos: Vec2) {
        match self {
            ConnectionInProgress::FromInput { src: _, dest } => *dest = pos.into(),
            ConnectionInProgress::FromOutput { src: _, dest } => *dest = pos.into(),
        }
    }
    /// If an output/input pair is created between src and dest, it is returned.
    /// # Example
    /// ```rust
    /// use yew_node_graph::{
    ///     state::{
    ///         ConnectTo, ConnectionInProgress, InputId, OutputId,
    ///         AnyParameterId
    ///     },
    ///     vec2,
    /// };
    /// fn example(input: InputId, output: OutputId) {
    ///     let mut connection_in_progress = ConnectionInProgress::FromInput {
    ///         src: input,
    ///         dest: ConnectTo::Id(output),
    ///     };
    ///
    ///     assert_eq!(
    ///         connection_in_progress.pair(),
    ///         Some((&output,&input))
    ///     );
    ///
    ///     // if dest is pos, return None.
    ///     assert_eq!(
    ///          ConnectionInProgress::FromInput {
    ///             src: input,
    ///             dest: ConnectTo::Pos(vec2(0.0,0.0)),
    ///         }.pair(),
    ///         None
    ///     );
    /// }
    /// ```
    pub fn pair(&self) -> Option<(&OutputId, &InputId)> {
        match self {
            ConnectionInProgress::FromInput {
                src: input,
                dest: ConnectTo::Id(output),
            } => Some((output, input)),
            ConnectionInProgress::FromOutput {
                src: output,
                dest: ConnectTo::Id(input),
            } => Some((output, input)),
            _ => None,
        }
    }

    ///　If an output/input pair is created between the argument and itself's src, it is returned.
    /// # Example
    /// ```rust
    /// use yew_node_graph::{
    ///     state::{
    ///         ConnectTo, ConnectionInProgress, InputId, OutputId,
    ///         AnyParameterId
    ///     },
    ///     vec2,
    /// };
    /// fn example(input: InputId, output: OutputId, another_input: InputId) {
    ///     let mut connection_in_progress = ConnectionInProgress::FromInput {
    ///         src: input,
    ///         dest: ConnectTo::Pos(vec2(12.0, 3.0)),
    ///     };
    ///
    ///     assert_eq!(
    ///         connection_in_progress.pair_with(AnyParameterId::Output(output)),
    ///         Some((output,input))
    ///     );
    ///     // Inputs cannot be connected to inputs.
    ///     assert_eq!(
    ///         connection_in_progress.pair_with(AnyParameterId::Input(another_input)),
    ///         None
    ///     )
    /// }
    /// ```
    pub fn pair_with(&self, id: AnyParameterId) -> Option<(OutputId, InputId)> {
        match (self, id) {
            (
                ConnectionInProgress::FromInput {
                    src: input,
                    dest: _,
                },
                AnyParameterId::Output(output),
            ) => Some((output, *input)),
            (
                ConnectionInProgress::FromOutput {
                    src: output,
                    dest: _,
                },
                AnyParameterId::Input(input),
            ) => Some((*output, input)),
            _ => None,
        }
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
/// What to do when a drag operation is performed.
/// If starting dragging from the background, it becomes [`DragState::SelectBox`],
/// if starting on a node, it becomes [`DragState::MoveNode`],
/// and if starting on a Port, it becomes [`DragState::ConnectPort`].
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum DragState {
    SelectBox {
        start: Vec2,
        end: Vec2,
    },
    Pan {
        prev_pos: Vec2,
    },
    MoveNode {
        id: NodeId,
        /// Mouse down position in node
        shift: Vec2,
    },
    ConnectPort(ConnectionInProgress),
}
