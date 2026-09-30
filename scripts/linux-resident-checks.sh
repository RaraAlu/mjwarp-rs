#!/usr/bin/env bash
run_resident_checks() {
    local name expected target filter mode
    local base=(cargo test --locked "${feature[@]}" "${profile[@]}")
    local ignore=()
    while IFS='|' read -r name expected target filter mode; do
        ignore=()
        if [[ $mode == ignored ]]; then ignore=(--ignored); fi
        if [[ $target == lib ]]; then
            run_checked "resident-$name" "$expected" "$expected" "${base[@]}" --lib "$filter" \
                -- "${ignore[@]}" --test-threads=1 --nocapture
        else
            run_checked "resident-$name" "$expected" "$expected" "${base[@]}" --test "$target" "$filter" \
                -- "${ignore[@]}" --test-threads=1 --nocapture
        fi
    done <<'CHECKS'
references|1|resident_kinematics|reference_manifests|normal
kinematics|6|resident_kinematics||ignored
parameters|3|resident_parameters||ignored
mocap-references|1|resident_mocap|reference_hashes|normal
mocap|4|resident_mocap||ignored
camlight-references|1|resident_camlight|reference_hashes|normal
camlight|5|resident_camlight||ignored
camlight-guards|1|lib|camlight_readback_rejects|ignored
fixed-references|1|resident_fixed_tendon|reference_hashes|normal
fixed|4|resident_fixed_tendon||ignored
fixed-guards|2|lib|fixed_tendon_readback_rejects|ignored
spatial-references|1|resident_spatial_tendon|reference_hashes|normal
spatial|5|resident_spatial_tendon||ignored
spatial-guards|2|lib|spatial_tendon_readback_rejects|ignored
geom-references|1|resident_geom_tendon|reference_hashes|normal
geom|6|resident_geom_tendon||ignored
global-references|1|resident_tendon|reference_hashes|normal
global-boundaries|1|resident_tendon|rejects_global|normal
global|3|resident_tendon||ignored
global-guards|1|lib|global_tendon_readback|ignored
position-references|1|resident_flex_position|reference_hashes|normal
position|4|resident_flex_position||ignored
position-guards|1|lib|flex_position_readback|ignored
wake-references|1|resident_tendon_wake|reference_hashes|normal
wake|5|resident_tendon_wake||ignored
wake-guards|1|lib|tendon_wake_readback|ignored
edge-references|1|resident_flex_edge|reference_hashes|normal
edge-boundaries|1|resident_flex_edge|rejects_invalid|normal
edge|4|resident_flex_edge||ignored
edge-guards|1|lib|flex_edge_readback|ignored
edge-free-body|1|lib|flex_edge_free_body|ignored
face-references|1|resident_flex_face|reference_hashes|normal
face-boundaries|1|resident_flex_face|rejects_face|normal
face|3|resident_flex_face||ignored
face-guards|1|lib|flex_face_readback|ignored
face-analytic|1|lib|flex_face_checks_analytic|ignored
hessian-layout|1|lib|checks_hessian_validity_layout|normal
hessian-decode|1|lib|flex_hessian_decodes|normal
hessian|3|lib|flex_hessian_|ignored
stretch-references|1|resident_flex_stretch|reference_hashes|normal
stretch-boundaries|1|resident_flex_stretch|rejects_hessian|normal
stretch-layout|1|lib|checks_hessian_layouts|normal
stretch|4|resident_flex_stretch||ignored
stretch-guards|2|lib|stretch_hessian_|ignored
static-cache|1|lib|static_geom_cache|ignored
adapter|5|lib|runtime::transfer::kernel::tests|ignored
CHECKS
    jq -e '[.[] | select(.name | startswith("resident-")) | .passed] | add == 98' \
        "$directory/checks.json" >/dev/null
}
