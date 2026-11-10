extern crate alloc;

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// The 8-node Hamon Cube hypercube topology (2^3 = 8 logical nodes).
/// Nodes are addressed with 3-bit coordinates from 0b000 (0) to 0b111 (7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeCoord(pub u8);

impl NodeCoord {
    /// Creates a node coordinate validated within 0..=7.
    pub const fn new(coord: u8) -> Self {
        Self(coord & 0b111)
    }

    /// Returns the 3 binary dimension bits [x, y, z].
    pub const fn bits(&self) -> [bool; 3] {
        [
            (self.0 & 0b001) != 0,
            (self.0 & 0b010) != 0,
            (self.0 & 0b100) != 0,
        ]
    }

    /// Computes the Hamming distance (number of differing bits) to another node.
    pub const fn hamming_distance(&self, other: &NodeCoord) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Returns the 3 direct 1-hop orthogonal neighbors on the hypercube.
    pub const fn direct_neighbors(&self) -> [NodeCoord; 3] {
        [
            NodeCoord(self.0 ^ 0b001),
            NodeCoord(self.0 ^ 0b010),
            NodeCoord(self.0 ^ 0b100),
        ]
    }

    /// Returns the neighbor along a specific hypercube axis `dim` (0 = X, 1 = Y, 2 = Z).
    pub const fn axis_neighbor(&self, dim: u8) -> NodeCoord {
        let mask = 1u8 << (dim & 0b11);
        NodeCoord((self.0 ^ mask) & 0b111)
    }

    /// Returns the antipodal opposite node coordinate across the hypercube diagonal.
    pub const fn antipode(&self) -> NodeCoord {
        NodeCoord(self.0 ^ 0b111)
    }
}

/// The Hamon Cube network manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HamonCubeTopology {
    /// Active 8-node logical hypercube addresses
    pub nodes: Vec<NodeCoord>,
}

impl HamonCubeTopology {
    /// Initializes the full 8-node Hamon Cube.
    pub fn new() -> Self {
        let nodes = (0..8).map(NodeCoord::new).collect();
        Self { nodes }
    }

    /// Computes the optimal next-hop routing step from `current` towards `destination`.
    pub fn next_hop(&self, current: NodeCoord, destination: NodeCoord) -> NodeCoord {
        if current == destination {
            return current;
        }

        let diff = current.0 ^ destination.0;
        // Select the lowest differing bit to step along the hypercube axis
        let bit_mask = 1 << (diff.trailing_zeros() & 0b11);
        NodeCoord(current.0 ^ bit_mask)
    }

    /// Computes the full shortest multi-hop path from `source` to `destination`
    /// (excluding `source`, including `destination`).
    pub fn route_path(&self, source: NodeCoord, destination: NodeCoord) -> Vec<NodeCoord> {
        let mut path = Vec::new();
        let mut curr = source;
        while curr != destination {
            curr = self.next_hop(curr, destination);
            path.push(curr);
        }
        path
    }
}
