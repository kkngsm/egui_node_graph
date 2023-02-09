use slotmap::{SecondaryMap, SlotMap};

/// Contains the main definitions for the node graph model.
pub mod graph;
pub use graph::*;

/// Type declarations for the different id types (node, input, output)
pub mod id_type;
pub use id_type::*;

/// Implements the index trait for the Graph type, allowing indexing by all
/// three id types
pub mod index_impls;

/// Implementing the main methods for the `Graph`
pub mod graph_impls;

/// Custom error types, crate-wide
pub mod error;
pub use error::*;

/// The main struct in the library, contains all the necessary state to draw the
/// UI graph
pub mod ui_state;
pub use ui_state::*;

/// Several traits that must be implemented by the user to customize the
/// behavior of this library.
pub mod traits;
pub use traits::*;
