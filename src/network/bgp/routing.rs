use crate::network::bgp::{BGPOrigin, RouteEntry, RouteTable};
use crate::node::{NodeTier, RoutePolicy};
use ipnet::IpNet;
use std::net::IpAddr;

pub struct RoutingPolicy {
    pub local_asn: u32,
    pub node_tier: NodeTier,
    pub route_policy: RoutePolicy,
    pub default_local_pref: u32,
    pub default_med: u32,
}

impl RoutingPolicy {
    pub fn new(local_asn: u32, node_tier: NodeTier) -> Self {
        let route_policy = node_tier.route_advertisement_policy();

        RoutingPolicy {
            local_asn,
            node_tier,
            route_policy,
            default_local_pref: 100,
            default_med: 0,
        }
    }

    /// Check if we should accept a route based on our tier policy
    pub fn should_accept_route(&self, route: &RouteEntry, peer_asn: u32) -> bool {
        let peer_tier = Self::asn_to_tier(peer_asn);

        match &self.route_policy {
            RoutePolicy::FullTable => {
                // Backbone nodes accept all routes from valid peers
                true
            }
            RoutePolicy::RegionalFilter => {
                // Regional nodes have more restrictive policies
                self.apply_regional_filter(route, peer_tier)
            }
            RoutePolicy::DefaultOnly => {
                // Edge nodes only accept default routes and local announcements
                self.is_default_route(route) || self.is_local_announcement(route, peer_asn)
            }
        }
    }

    /// Check if we should advertise a route to a peer
    pub fn should_advertise_route(&self, route: &RouteEntry, peer_asn: u32) -> bool {
        let peer_tier = Self::asn_to_tier(peer_asn);

        match &self.route_policy {
            RoutePolicy::FullTable => {
                // Backbone advertises all routes (with loop prevention)
                !self.has_asn_loop(route, peer_asn)
            }
            RoutePolicy::RegionalFilter => {
                // Regional nodes filter what they advertise
                self.apply_regional_advertisement_filter(route, peer_tier)
            }
            RoutePolicy::DefaultOnly => {
                // Edge nodes only advertise local services
                self.is_local_route(route)
            }
        }
    }

    fn apply_regional_filter(&self, route: &RouteEntry, peer_tier: NodeTier) -> bool {
        match peer_tier {
            NodeTier::Backbone => true, // Accept all from backbone
            NodeTier::Regional => {
                // Accept regional routes and local services
                route.as_path.len() <= 3 // Limit path length
            }
            NodeTier::Edge => {
                // Only accept local service announcements from edge
                self.is_local_service_route(route)
            }
        }
    }

    fn apply_regional_advertisement_filter(&self, route: &RouteEntry, peer_tier: NodeTier) -> bool {
        match peer_tier {
            NodeTier::Backbone => {
                // Advertise aggregated routes to backbone
                self.is_aggregatable_route(route)
            }
            NodeTier::Regional => {
                // Share routes with other regionals
                !self.has_asn_loop(route, 0) // General loop prevention
            }
            NodeTier::Edge => {
                // Send default route + reachable services to edge
                self.is_default_route(route) || self.is_reachable_service(route)
            }
        }
    }

    fn asn_to_tier(asn: u32) -> NodeTier {
        match asn {
            65000..=65099 => NodeTier::Backbone,
            65100..=65999 => NodeTier::Regional,
            66000..=69999 => NodeTier::Edge,
            _ => NodeTier::Edge,
        }
    }

    fn is_default_route(&self, route: &RouteEntry) -> bool {
        route.network == "0.0.0.0/0".parse().unwrap()
            || route.network == "10.0.0.0/8".parse().unwrap() // VX0 default
    }

    fn is_local_route(&self, route: &RouteEntry) -> bool {
        route.as_path.first() == Some(&self.local_asn)
    }

    fn is_local_announcement(&self, route: &RouteEntry, peer_asn: u32) -> bool {
        route.as_path == vec![peer_asn] // Direct announcement from peer
    }

    fn is_local_service_route(&self, route: &RouteEntry) -> bool {
        // Check if this is a service route (typically /32 or small subnets)
        route.network.prefix_len() >= 24
    }

    fn is_aggregatable_route(&self, route: &RouteEntry) -> bool {
        // Routes that can be aggregated for backbone advertisement
        route.network.prefix_len() <= 16 // Only larger prefixes
    }

    fn is_reachable_service(&self, route: &RouteEntry) -> bool {
        // Services that should be advertised to edge nodes
        route.network.prefix_len() >= 24 && route.local_pref >= 100
    }

    fn has_asn_loop(&self, route: &RouteEntry, peer_asn: u32) -> bool {
        route.as_path.contains(&peer_asn)
    }

    pub fn evaluate_route(&self, route: &RouteEntry) -> u32 {
        // Simple route preference calculation
        // Higher values indicate better routes
        let mut preference = 0;

        // Prefer routes with higher local preference
        preference += route.local_pref;

        // Prefer routes with shorter AS path
        if !route.as_path.is_empty() {
            preference += 100 / route.as_path.len() as u32;
        }

        // Prefer IGP origin over EGP/Incomplete
        match route.origin {
            BGPOrigin::IGP => preference += 10,
            BGPOrigin::EGP => preference += 5,
            BGPOrigin::Incomplete => preference += 0,
        }

        preference
    }

    pub fn select_best_route(&self, routes: &[RouteEntry]) -> Option<RouteEntry> {
        if routes.is_empty() {
            return None;
        }

        let mut best_route = &routes[0];
        let mut best_preference = self.evaluate_route(best_route);

        for route in routes.iter().skip(1) {
            let preference = self.evaluate_route(route);
            if preference > best_preference {
                best_route = route;
                best_preference = preference;
            }
        }

        Some(best_route.clone())
    }
}

impl RouteTable {
    pub fn find_best_route(&self, destination: &IpAddr) -> Option<&RouteEntry> {
        // Find the most specific route (longest prefix match)
        let mut best_route = None;
        let mut best_prefix_len = 0;

        for (network, route) in &self.routes {
            if network.contains(destination) {
                let prefix_len = network.prefix_len();
                if prefix_len > best_prefix_len {
                    best_route = Some(route);
                    best_prefix_len = prefix_len;
                }
            }
        }

        best_route
    }

    pub fn get_routes_for_prefix(&self, network: &IpNet) -> Vec<&RouteEntry> {
        self.routes
            .values()
            .filter(|route| route.network == *network)
            .collect()
    }

    pub fn announce_vx0_network(
        &mut self,
        vx0_network: IpNet,
        local_asn: u32,
    ) -> Result<(), crate::network::bgp::BGPError> {
        let route = RouteEntry {
            network: vx0_network,
            next_hop: "10.0.0.1".parse().unwrap(), // VX0 gateway
            as_path: vec![local_asn],
            origin: BGPOrigin::IGP,
            local_pref: 200, // High preference for VX0 routes
            med: 0,
            communities: vec![],
            timestamp: chrono::Utc::now(),
        };

        self.add_route(route)?;
        tracing::info!("Announced VX0 network: {}", vx0_network);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_evaluation() {
        let policy = RoutingPolicy::new(65001, crate::node::NodeTier::Edge);

        let route = RouteEntry {
            network: "10.0.0.0/24".parse().unwrap(),
            next_hop: "192.168.1.1".parse().unwrap(),
            as_path: vec![65001, 65002],
            origin: BGPOrigin::IGP,
            local_pref: 100,
            med: 0,
            communities: vec![],
            timestamp: chrono::Utc::now(),
        };

        let preference = policy.evaluate_route(&route);
        assert!(preference > 0);
    }

    #[test]
    fn test_best_route_selection() {
        let policy = RoutingPolicy::new(65001, crate::node::NodeTier::Edge);

        let route1 = RouteEntry {
            network: "10.0.0.0/24".parse().unwrap(),
            next_hop: "192.168.1.1".parse().unwrap(),
            as_path: vec![65001, 65002],
            origin: BGPOrigin::IGP,
            local_pref: 100,
            med: 0,
            communities: vec![],
            timestamp: chrono::Utc::now(),
        };

        let route2 = RouteEntry {
            network: "10.0.0.0/24".parse().unwrap(),
            next_hop: "192.168.1.2".parse().unwrap(),
            as_path: vec![65001, 65003, 65004],
            origin: BGPOrigin::EGP,
            local_pref: 150,
            med: 0,
            communities: vec![],
            timestamp: chrono::Utc::now(),
        };

        let routes = vec![route1, route2];
        let best = policy.select_best_route(&routes);

        assert!(best.is_some());
        assert_eq!(best.unwrap().local_pref, 150);
    }
}
