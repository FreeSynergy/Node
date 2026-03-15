// Node discovery — find FSN nodes on the local network.

/// A FreeSynergy.Node discovered on the local network.
#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    /// Short hostname of the discovered node (e.g. "node2").
    pub hostname: String,
    /// IP address of the discovered node.
    pub ip: String,
    /// Cluster ID that the node belongs to.
    pub cluster_id: String,
    /// FSN version string reported by the node.
    pub version: String,
}

impl DiscoveredNode {
    /// Create a new `DiscoveredNode`.
    pub fn new(
        hostname: impl Into<String>,
        ip: impl Into<String>,
        cluster_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            ip: ip.into(),
            cluster_id: cluster_id.into(),
            version: version.into(),
        }
    }
}

// ── Discovery trait ───────────────────────────────────────────────────────────

/// Backend for discovering FSN nodes.
///
/// Implementations: `MdnsDiscovery`, `ManualDiscovery`.
pub trait NodeDiscovery {
    /// Discover and return all reachable FSN nodes.
    fn discover(&self) -> Vec<DiscoveredNode>;
}

// ── ManualDiscovery ───────────────────────────────────────────────────────────

/// Discovery backend that returns a pre-configured list of nodes.
///
/// Used when mDNS is unavailable or in test scenarios.
pub struct ManualDiscovery {
    /// Nodes to return on every `discover()` call.
    pub nodes: Vec<DiscoveredNode>,
}

impl ManualDiscovery {
    /// Create a new `ManualDiscovery` with the given nodes.
    pub fn new(nodes: Vec<DiscoveredNode>) -> Self {
        Self { nodes }
    }

    /// Create an empty `ManualDiscovery`.
    pub fn empty() -> Self {
        Self { nodes: vec![] }
    }

    /// Add a node to the manual list.
    pub fn add(&mut self, node: DiscoveredNode) {
        self.nodes.push(node);
    }
}

impl NodeDiscovery for ManualDiscovery {
    fn discover(&self) -> Vec<DiscoveredNode> {
        self.nodes.clone()
    }
}

// ── MdnsDiscovery ─────────────────────────────────────────────────────────────

/// mDNS-based node discovery (stub — production would use the `mdns` or `zeroconf` crate).
///
/// Currently returns an empty list. Replace `discover()` with real mDNS probing
/// once the crate dependency is added.
pub struct MdnsDiscovery;

impl MdnsDiscovery {
    /// Create a new `MdnsDiscovery` instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MdnsDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeDiscovery for MdnsDiscovery {
    fn discover(&self) -> Vec<DiscoveredNode> {
        // TODO: implement real mDNS discovery using the `mdns` or `zeroconf` crate.
        vec![]
    }
}
