use klm::swarm::{HamonCubeTopology, NodeCoord, SwarmOrchestrator};

#[test]
fn test_hamon_cube_distance_and_routing() {
    let topology = HamonCubeTopology::new();
    let n0 = NodeCoord::new(0b000);
    let n7 = NodeCoord::new(0b111);

    assert_eq!(n0.hamming_distance(&n7), 3);
    assert_eq!(n0.antipode(), n7);

    let next = topology.next_hop(n0, n7);
    assert_eq!(next.hamming_distance(&n0), 1);
}

#[test]
fn test_swarm_interaction_and_dream_rotation() {
    let mut swarm = SwarmOrchestrator::new();
    let (resp, activated) = swarm.interact("Hello to my family", 0.9);
    assert!(!resp.is_empty());
    assert!(!activated.is_empty());

    let dream_records = swarm.process_dreams();
    assert!(!dream_records.is_empty());

    swarm.rotate_circadian_cohorts();
    assert_eq!(swarm.global_epoch, 1);
}
