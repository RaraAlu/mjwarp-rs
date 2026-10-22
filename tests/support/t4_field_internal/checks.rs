use super::*;

pub(super) fn validate(i: &Inventory) -> Check {
    require(
        i.schema == 1 && i.scope == "g01-batched-parameters-and-com-internals",
        "scope",
    )?;
    require(
        i.product_revision == "dcb1fae771c45ca5df823d0b2517f04724f625b2"
            && i.upstream_revision == super::super::REVISION,
        "revision",
    )?;
    require(
        !i.np01_complete && i.remaining.iter().map(String::as_str).eq(REMAINING),
        "remaining work",
    )?;
    let paths: BTreeSet<_> = i.product_sources.iter().map(|s| s.path.as_str()).collect();
    require(
        paths.len() == i.product_sources.len()
            && paths == SOURCES.iter().map(|(p, _)| *p).collect(),
        "source coverage",
    )?;
    for stamp in &i.product_sources {
        require(
            digest(source(&stamp.path)?) == stamp.sha256,
            format!("source drift: {}", stamp.path),
        )?;
    }
    require(
        i.upstream_smooth.path == "_src/smooth.py"
            && i.upstream_smooth.sha256
                == "6268f0e1b6cb4c9ec6bbd3aa29828dfb36224efb2a3a3e1f54e5f87b6623fdef",
        "smooth source",
    )?;
    let input = super::super::inventory();
    let mut expected = Vec::new();
    for (group, start) in [
        ("rigid", "const RIGID_PARAMETERS:"),
        ("com", "const COM_PARAMETERS:"),
        ("attached", "const ATTACHED_PARAMETERS:"),
    ] {
        for (slot, variant) in variants(RESIDENT, start, "KinematicsParameter::")?
            .into_iter()
            .enumerate()
        {
            expected.push((group, slot, variant));
        }
    }
    for (slot, variant) in variants(CAMLIGHT, "pub const ALL:", "Self::")?
        .into_iter()
        .enumerate()
    {
        expected.push(("camlight", slot, variant));
    }
    require(
        expected.len() == 23 && i.fields.len() == 23,
        "field coverage",
    )?;
    // 同时覆盖公开枚举，避免新增字段静默漏审。
    let public: BTreeSet<_> = mjwarp_rs::model::KinematicsParameter::ALL
        .into_iter()
        .map(|f| f.name())
        .chain(
            mjwarp_rs::model::CamLightParameter::ALL
                .into_iter()
                .map(|f| f.name()),
        )
        .collect();
    require(
        public.len() == 23
            && i.fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<BTreeSet<_>>()
                == public,
        "public field coverage",
    )?;
    for (f, (group, slot, variant)) in i.fields.iter().zip(expected) {
        require(
            f.group == group && f.slot == slot && f.variant == variant,
            "packing order",
        )?;
        require(
            f.descriptor_scalar == "i32" && f.parameter_scalar == "f32",
            "packed scalar",
        )?;
        let declaration = input
            .fields
            .iter()
            .find(|p| p.name == f.name)
            .ok_or("unknown input")?;
        require(
            f.upstream_declaration == declaration.upstream_declaration
                && super::super::upstream_scalar(&f.upstream_declaration)? == "f32",
            "upstream field scalar",
        )?;
        let host = if group == "camlight" {
            CAMLIGHT
        } else {
            PARAMETERS
        };
        require(
            host.contains(&format!("Self::{} => \"{}\",", f.variant, f.name)),
            "host field identity",
        )?;
        let shared = between(host, "pub(crate) fn shared(", "\n}")?;
        require(
            shared.lines().any(|l| {
                l.trim().starts_with(&format!("Self::{} => &", f.variant))
                    && l.trim().ends_with(&format!(".{},", f.name))
            }),
            "shared field source",
        )?;
        let entry = match group {
            "rigid" => "rigid_kinematics",
            "com" => "rigid_com_position",
            "attached" => "attached_kinematics",
            _ if matches!(slot, 2 | 7) => "camlight_com",
            _ => "camlight_rigid",
        };
        require(f.entry == entry && !f.reader.is_empty(), "reader routing")?;
        require(
            kernel(entry)?.0.contains(&reader_anchor(f)?),
            "descriptor reader",
        )?;
    }
    for (start, end) in [
        ("fn pack_model(", "// Descriptors hold"),
        ("fn pack_camlight(", "impl PackedModel"),
    ] {
        let pack = between(RESIDENT, start, end)?;
        for anchor in [
            "host_staging::<i32>(metadata_len)",
            "host_staging::<f32>(parameters_len.max(1))",
            "metadata[cursor] = offset as i32;",
            "metadata[cursor + 1] = parameters.batches(f) as i32;",
            "values[offset..offset + v.len()].copy_from_slice(v);",
            "offset += v.len();",
            "cursor += 2;",
        ] {
            require(pack.contains(anchor), "typed pack route")?;
        }
    }
    for anchor in [
        "parameters: session.upload(&parameters)?",
        "return p+desc[0]+(unsigned long long)(world % (unsigned)desc[1])*width;",
    ] {
        require(RESIDENT.contains(anchor), "upload and independent period")?;
    }
    validate_com(i, com_source())
}

pub(super) fn validate_com(i: &Inventory, text: &str) -> Check {
    require(i.com_steps.len() == STEPS.len(), "COM step coverage")?;
    // 哈希绑定整个模板，局部锚点只提供可读映射。
    require(digest(text) == digest(com_source()), "COM template drift")?;
    for (step, &(id, symbol, classification, ty)) in i.com_steps.iter().zip(STEPS) {
        require(
            (
                step.id.as_str(),
                step.symbol.as_str(),
                step.classification.as_str(),
                step.upstream_type.as_str(),
            ) == (id, symbol, classification, ty)
                && step.scalar == "f32",
            "COM type or boundary",
        )?;
        require(
            !step.product.is_empty() && !step.upstream.is_empty(),
            "COM missing anchors",
        )?;
        for anchor in &step.product {
            require(
                !anchor.trim().is_empty() && text.contains(anchor),
                "COM product anchor",
            )?;
        }
    }
    Ok(())
}

pub(super) fn verify_upstream(i: &Inventory, smooth: &str, types: &str, warp: &str) -> Check {
    require(
        digest(smooth) == i.upstream_smooth.sha256,
        "upstream smooth drift",
    )?;
    super::super::verify_external(&super::super::inventory(), types, warp)?;
    for anchor in [
        "class vec10f(wp.types.vector(length=10, dtype=float)):",
        "vec10 = vec10f",
    ] {
        require(types.contains(anchor), "inertia type")?;
    }
    for anchor in [
        "class spatial_vectorf(vector(length=6, dtype=float32)):",
        "spatial_vector = spatial_vectorf",
        "class mat33f(matrix(shape=(3, 3), dtype=float32)):",
        "mat33 = mat33f",
    ] {
        require(warp.contains(anchor), "internal scalar type")?;
    }
    for step in &i.com_steps {
        let body = smooth
            .split_once(&format!("def {}(", step.symbol))
            .ok_or("upstream symbol")?
            .1;
        let body = body.split("\n\n\n").next().ok_or("upstream body")?;
        for anchor in &step.upstream {
            require(
                !anchor.trim().is_empty() && body.contains(anchor),
                format!("upstream expression: {}", step.id),
            )?;
        }
    }
    Ok(())
}

pub(super) fn verify_captures(i: &Inventory, root: &Path) -> Result<(usize, usize), String> {
    let manifest: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/precision/t4-kernels.json"))
            .map_err(|e| e.to_string())?;
    let artifacts = manifest["artifacts"].as_array().ok_or("artifacts")?;
    let mut total = 0;
    let mut batched = 0;
    for mode in ["debug-ptx", "release-ptx"] {
        let mut entries = BTreeSet::new();
        let mut fields = BTreeSet::new();
        for artifact in artifacts {
            let entry = artifact["entry"].as_str().ok_or("entry")?;
            if kernel(entry).is_err() {
                continue;
            }
            let ptx = artifact["ptx_sha256"].as_str().ok_or("PTX hash")?;
            require(
                ptx.len() == 64 && ptx.bytes().all(|b| b.is_ascii_hexdigit()),
                "capture path",
            )?;
            let stem = format!("{entry}-{ptx}");
            let mut cuda = String::new();
            for (ext, hash_key) in [
                ("cu", "source_sha256"),
                ("ptx", "ptx_sha256"),
                ("json", "receipt_sha256"),
            ] {
                let bytes = std::fs::read(root.join(mode).join(format!("{stem}.{ext}")))
                    .map_err(|e| e.to_string())?;
                let expected = artifact[hash_key].as_str().ok_or("artifact hash")?;
                require(
                    format!("{:x}", Sha256::digest(&bytes)) == expected,
                    "artifact drift",
                )?;
                if ext == "cu" {
                    cuda = std::str::from_utf8(&bytes)
                        .map_err(|e| e.to_string())?
                        .to_owned();
                }
                if ext == "json" {
                    let receipt: serde_json::Value =
                        serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                    require(
                        receipt["entry"] == entry
                            && receipt["passed"] == true
                            && receipt["precisionGate"] == "f32-physics"
                            && receipt["fmad"] == true
                            && receipt["useFastMath"] == false,
                        "precision receipt",
                    )?;
                }
            }
            if entry == "rigid_com_position" {
                require(
                    cuda.replace("\r\n", "\n").contains(com_source()),
                    "captured COM template",
                )?;
            }
            // 只给真正启用批量宏的产物记字段覆盖。
            if cuda
                .lines()
                .any(|l| l.trim() == "#define MJWARP_FIELD_BATCHES")
            {
                for f in i.fields.iter().filter(|f| f.entry == entry) {
                    require(cuda.contains(&reader_anchor(f)?), "captured field reader")?;
                    fields.insert(f.name.as_str());
                }
                batched += 1;
            }
            entries.insert((entry, ptx));
            total += 1;
        }
        require(
            entries.len() == 10 && fields.len() == 23,
            "capture coverage",
        )?;
    }
    Ok((total, batched))
}
