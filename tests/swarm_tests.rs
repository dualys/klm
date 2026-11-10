use klm::swarm::{BrainSpecialization, HamonCubeTopology, NodeCoord, SwarmOrchestrator};

#[test]
fn test_hamon_cube_distance_routing_and_specializations() {
    let topology = HamonCubeTopology::new();
    let n0 = NodeCoord::new(0b000);
    let n7 = NodeCoord::new(0b111);

    assert_eq!(n0.hamming_distance(&n7), 3);
    assert_eq!(n0.antipode(), n7);

    let path = topology.route_path(n0, n7);
    assert_eq!(path.len(), 3);
    assert_eq!(*path.last().unwrap(), n7);

    assert_eq!(
        BrainSpecialization::from_coord(n0),
        BrainSpecialization::RelationalEmpathy
    );
    assert_eq!(
        BrainSpecialization::from_coord(n7),
        BrainSpecialization::SovereignGuardian
    );
}

#[test]
fn test_swarm_interaction_dream_rotation_and_distributed_kpack_job() {
    let mut swarm = SwarmOrchestrator::new();
    let (resp, activated) = swarm.interact("Hello to my family", 0.9);
    assert!(!resp.is_empty());
    assert!(!activated.is_empty());

    let dream_records = swarm.process_dreams();
    assert!(!dream_records.is_empty());

    swarm.rotate_circadian_cohorts();
    assert_eq!(swarm.global_epoch, 1);

    // Execute distributed Kpack job across the 8 logical nodes of the Hamon Cube
    let payload = b"Distributed Kpack Job across 8 logical Hamon Cube nodes (000..111) with 3D butterfly Merkle reduction!";
    let job = swarm.execute_distributed_kpack_job(payload);
    assert_eq!(job.shards.len(), 8);
    assert_ne!(job.cube_merkle_root, [0u8; 32]);

    let reconstructed = job
        .gather_and_verify()
        .expect("8-node gather and 3D Merkle verification should succeed");
    assert_eq!(&reconstructed[..], &payload[..]);
}
