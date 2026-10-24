// QA independent harness for the S2a scene-BVH semantics.
// Builds the BVH from the committed fixture AABBs (already bit-checked against the
// upstream reference by the Python recomputation) and independently verifies:
//   - build/refit invariants (bounds equality, reachability, union/root semantics)
//   - the changed-leaf set after refit
//   - two-world grouping plus 32x2 semantic queries (no cross-world candidates)
//   - the rejection surface
// It uses the product crate at commit f9adf14 through a separate git worktree.

mod fixture_data;

use fixture_data::{CASES, ENABLED_GEOM_IDS, NGEOM};
use mjwarp_rs::render::bvh::{
    Aabb, LeafGeometry, NodeKind, SceneBvh, SceneBvhInput, build_scene_bvh, compute_leaf_aabb,
    mesh_half_extents, refit_bvh,
};

fn bits(value: u32) -> f32 {
    f32::from_bits(value)
}

fn case_bounds(index: usize) -> Vec<Aabb> {
    let case = &CASES[index];
    (0..NGEOM)
        .map(|g| Aabb {
            lower: [bits(case.lower[g][0]), bits(case.lower[g][1]), bits(case.lower[g][2])],
            upper: [bits(case.upper[g][0]), bits(case.upper[g][1]), bits(case.upper[g][2])],
        })
        .collect()
}

fn union_of(bounds: &[Aabb]) -> Aabb {
    bounds.iter().copied().reduce(Aabb::union).unwrap()
}

fn invariants(bvh: &SceneBvh, expected: &[Aabb], label: &str, errors: &mut Vec<String>) {
    if bvh.leaf_count() != expected.len() {
        errors.push(format!("{label}: leaf_count {} != {}", bvh.leaf_count(), expected.len()));
    }
    if bvh.leaf_bounds() != expected {
        errors.push(format!("{label}: leaf bounds differ from the fixture"));
    }
    if bvh.worlds() != 1 {
        errors.push(format!("{label}: worlds {} != 1", bvh.worlds()));
    }
    let root = match bvh.group_root(0) {
        Ok(root) => root,
        Err(error) => {
            errors.push(format!("{label}: group_root error {error:?}"));
            return;
        }
    };
    let nodes = bvh.nodes();
    if nodes[root as usize].bounds != union_of(expected) {
        errors.push(format!("{label}: root bounds != union of leaves"));
    }
    let mut visited = vec![0usize; expected.len()];
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        let node = nodes[index as usize];
        match node.kind {
            NodeKind::Leaf { leaf } => {
                visited[leaf as usize] += 1;
                if node.bounds != expected[leaf as usize] {
                    errors.push(format!("{label}: leaf {leaf} node bounds differ"));
                }
            }
            NodeKind::Internal { left, right } => {
                let union = nodes[left as usize].bounds.union(nodes[right as usize].bounds);
                if node.bounds != union {
                    errors.push(format!("{label}: internal node {index} bounds != union(children)"));
                }
                stack.push(left);
                stack.push(right);
            }
        }
    }
    for (leaf, count) in visited.iter().enumerate() {
        if *count != 1 {
            errors.push(format!("{label}: leaf {leaf} reached {count} times"));
        }
    }
}

fn main() {
    let mut out = String::new();
    let mut failures = 0usize;
    let initial = case_bounds(0);
    let moved = case_bounds(1);

    // 1) build + refit invariants over all 8 poses
    let groups = vec![0u32; NGEOM];
    let mut bvh = build_scene_bvh(SceneBvhInput {
        bounds: &initial,
        groups: &groups,
        geom_ids: &ENABLED_GEOM_IDS,
        worlds: 1,
    })
    .expect("build initial bvh");
    let mut errors = Vec::new();
    invariants(&bvh, &initial, "initial", &mut errors);
    refit_bvh(&mut bvh, &moved).expect("refit moved");
    invariants(&bvh, &moved, "moved", &mut errors);
    for pose in 2..CASES.len() {
        let bounds = case_bounds(pose);
        refit_bvh(&mut bvh, &bounds).expect("refit pose");
        invariants(&bvh, &bounds, CASES[pose].name, &mut errors);
    }
    let changed: Vec<usize> = (0..NGEOM).filter(|leaf| initial[*leaf] != moved[*leaf]).collect();
    if changed != vec![1, 2, 3, 4, 5, 6] {
        errors.push(format!("changed leaves {changed:?} != [1,2,3,4,5,6]"));
    }
    failures += errors.len();
    out.push_str(&format!(
        "build_refit: poses={} errors={} changed_leaves={:?}\n",
        CASES.len(),
        errors.len(),
        changed
    ));
    for error in &errors {
        out.push_str(&format!("  build_refit_error: {error}\n"));
    }

    // 2) two worlds, 32 rays x 2 worlds
    let mut duplicated = initial.clone();
    duplicated.extend_from_slice(&initial);
    let mut two_groups = vec![0u32; NGEOM];
    two_groups.extend(vec![1u32; NGEOM]);
    let mut two_geom_ids = ENABLED_GEOM_IDS.to_vec();
    two_geom_ids.extend_from_slice(&ENABLED_GEOM_IDS);
    let two = build_scene_bvh(SceneBvhInput {
        bounds: &duplicated,
        groups: &two_groups,
        geom_ids: &two_geom_ids,
        worlds: 2,
    })
    .expect("build two-world bvh");
    let center = union_of(&initial).center();
    let floor_top = initial[0].upper[2];
    let mut hits = 0usize;
    let mut q_errors = 0usize;
    for index in 0..32usize {
        let theta = index as f32 * (std::f32::consts::TAU / 32.0);
        let radius = 0.5 + (index % 5) as f32 * 0.5;
        let origin = [
            center[0] + radius * theta.cos(),
            center[1] + radius * theta.sin(),
            floor_top + 5.0,
        ];
        let direction = [center[0] - origin[0], center[1] - origin[1], center[2] - origin[2]];
        let mut per_world = Vec::new();
        for world in 0..2usize {
            let candidates = two.ray_leaf_candidates(world, origin, direction).expect("query");
            if !candidates.is_empty() {
                hits += 1;
            }
            let mut geom_hits = Vec::new();
            for candidate in &candidates {
                if two.leaf_group(*candidate) != Some(world as u32) {
                    q_errors += 1;
                }
                geom_hits.push(two.leaf_geom_id(*candidate).expect("geom id"));
            }
            geom_hits.sort_unstable();
            per_world.push(geom_hits);
        }
        if per_world[0] != per_world[1] {
            q_errors += 1;
        }
    }
    // QA-owned extra ray set: 24 deterministic shallow rays + 8 upward misses
    let mut extra_hits = 0usize;
    let mut extra_queries = 0usize;
    for index in 0..24usize {
        let theta = (index as f32) * (std::f32::consts::TAU / 24.0) * 0.5;
        let radius = 2.0 + (index % 4) as f32;
        let origin = [center[0] + radius, center[1] + radius * theta.sin(), center[2] + 2.0];
        let direction = [center[0] - origin[0], center[1] - origin[1], center[2] - origin[2]];
        for world in 0..2usize {
            let candidates = two.ray_leaf_candidates(world, origin, direction).expect("extra query");
            extra_queries += 1;
            if !candidates.is_empty() {
                extra_hits += 1;
            }
            for candidate in &candidates {
                if two.leaf_group(*candidate) != Some(world as u32) {
                    q_errors += 1;
                }
            }
        }
    }
    let mut miss_hits = 0usize;
    for index in 0..8usize {
        let origin = [center[0] + index as f32 * 0.1, center[1], center[2] + 5.0];
        for world in 0..2usize {
            let candidates = two
                .ray_leaf_candidates(world, origin, [0.0, 0.0, 1.0])
                .expect("miss query");
            miss_hits += candidates.len();
        }
    }
    failures += q_errors;
    out.push_str(&format!(
        "two_world: rays=32 worlds=2 hits={}/64 extra_queries={} extra_hits={} upward_miss_candidates={} cross_world_or_set_errors={}\n",
        hits, extra_queries, extra_hits, miss_hits, q_errors
    ));

    // 3) rejection surface
    let empty: [Aabb; 0] = [];
    let mut rejections = Vec::new();
    let mut record = |name: &str, rejected: bool, rejections: &mut Vec<(String, bool)>, out: &mut String| {
        rejections.push((name.to_string(), rejected));
        out.push_str(&format!("reject: {name} rejected={rejected}\n"));
    };
    let attempt = |bounds: &[Aabb], groups: &[u32], geom_ids: &[i32], worlds: usize| {
        build_scene_bvh(SceneBvhInput { bounds, groups, geom_ids, worlds })
    };
    record("empty_scene", attempt(&empty, &[], &[], 1).is_err(), &mut rejections, &mut out);
    record(
        "groups_length_mismatch",
        attempt(&initial, &groups[..2], &ENABLED_GEOM_IDS, 1).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "geom_ids_length_mismatch",
        attempt(&initial, &groups, &ENABLED_GEOM_IDS[..3], 1).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "zero_worlds",
        attempt(&initial, &groups, &ENABLED_GEOM_IDS, 0).is_err(),
        &mut rejections,
        &mut out,
    );
    let mut out_of_range = groups.clone();
    out_of_range[3] = 5;
    record(
        "group_out_of_range",
        attempt(&initial, &out_of_range, &ENABLED_GEOM_IDS, 1).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "empty_world",
        attempt(&initial, &groups, &ENABLED_GEOM_IDS, 2).is_err(),
        &mut rejections,
        &mut out,
    );
    let mut invalid = initial.clone();
    invalid[2] = Aabb { lower: [1.0; 3], upper: [-1.0; 3] };
    record(
        "invalid_aabb_build",
        attempt(&invalid, &groups, &ENABLED_GEOM_IDS, 1).is_err(),
        &mut rejections,
        &mut out,
    );
    let mut buildable = attempt(&initial, &groups, &ENABLED_GEOM_IDS, 1).expect("buildable");
    record(
        "refit_length_mismatch",
        refit_bvh(&mut buildable, &initial[..3]).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "refit_invalid_aabb",
        refit_bvh(&mut buildable, &invalid).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "unsupported_geom_type_hfield",
        LeafGeometry::from_geom_type(1, [1.0, 1.0, 1.0]).is_err(),
        &mut rejections,
        &mut out,
    );
    let plane = LeafGeometry::from_geom_type(0, [1.0, 1.0, 1.0]).expect("plane");
    record(
        "non_finite_pose",
        compute_leaf_aabb(
            &plane,
            [f32::NAN, 0.0, 0.0],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        )
        .is_err(),
        &mut rejections,
        &mut out,
    );
    record("empty_mesh_bounds", mesh_half_extents(&[]).is_err(), &mut rejections, &mut out);
    record(
        "aabb_new_non_finite",
        Aabb::new([f32::NAN, 0.0, 0.0], [1.0, 1.0, 1.0]).is_err(),
        &mut rejections,
        &mut out,
    );
    record(
        "world_out_of_range",
        two.ray_leaf_candidates(2, center, [0.0, 0.0, -1.0]).is_err(),
        &mut rejections,
        &mut out,
    );
    let rejected = rejections.iter().filter(|(_, ok)| *ok).count();
    if rejected != rejections.len() {
        failures += rejections.len() - rejected;
    }
    out.push_str(&format!("rejections: {}/{} rejected\n", rejected, rejections.len()));

    out.push_str(&format!(
        "RESULT failures={} build_refit_errors={} query_errors={} rejected={}/{} hits={}\n",
        failures,
        errors.len(),
        q_errors,
        rejected,
        rejections.len(),
        hits
    ));
    print!("{out}");
    std::fs::write("qa-s2a-harness-result.txt", out).expect("write result");
    if failures != 0 {
        std::process::exit(1);
    }
}
