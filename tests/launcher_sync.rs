use mjwarp_rs::runtime::launcher::{
    Access, BlockRule, Checked, DeviceSpan, ElemType, Kernel, KernelAbi, KernelSpec, LauncherError,
    MetadataProof, MetadataRevision, ProofKind, SlotBinding, SlotDesc, SlotKind, SlotRequirement,
    SlotRole,
};

struct Metadata;

impl MetadataProof for Metadata {
    type Dims = u32;

    const KIND: ProofKind = ProofKind("index");

    fn validate(values: &[i32], dims: &Self::Dims) -> Result<(), LauncherError> {
        if values.len() != 2 || values[0] != *dims as i32 || values[1] < 0 {
            return Err(LauncherError::InvalidMetadata("测试索引越界"));
        }
        Ok(())
    }
}

struct Example;

unsafe impl KernelAbi for Example {
    const NAME: &'static str = "example";
    const BLOCK: BlockRule = BlockRule::Fixed(64);
    const SLOTS: &'static [SlotDesc] = &[
        SlotDesc {
            index: 0,
            kind: SlotKind::DevicePtr {
                elem: ElemType::I32,
            },
            access: Access::Read,
            role: SlotRole::AddressBearing {
                proof: ProofKind("index"),
            },
        },
        SlotDesc {
            index: 1,
            kind: SlotKind::DevicePtr {
                elem: ElemType::I32,
            },
            access: Access::Read,
            role: SlotRole::AddressBearing {
                proof: ProofKind("index"),
            },
        },
        SlotDesc {
            index: 2,
            kind: SlotKind::DevicePtr {
                elem: ElemType::F32,
            },
            access: Access::Write,
            role: SlotRole::Value,
        },
    ];
}

impl KernelSpec for Example {
    type Dims = u32;
    fn spans(dims: &Self::Dims) -> Result<Vec<SlotRequirement>, LauncherError> {
        Ok(vec![
            SlotRequirement {
                slot: 0,
                min_elements: 2,
            },
            SlotRequirement {
                slot: 1,
                min_elements: u64::from(*dims),
            },
            SlotRequirement {
                slot: 2,
                min_elements: u64::from(*dims),
            },
        ])
    }

    fn launch_dims(
        dims: &Self::Dims,
    ) -> Result<mjwarp_rs::runtime::launcher::LaunchDims, LauncherError> {
        mjwarp_rs::runtime::launcher::LaunchDims::new((dims.div_ceil(64), 1, 1), (64, 1, 1), 0)
    }
}

fn span(context: u64, address: u64, bytes: u64) -> DeviceSpan {
    DeviceSpan::new(context, address, bytes).unwrap()
}

fn binding(slot: usize, context: u64, address: u64, bytes: u64, elem: ElemType) -> SlotBinding {
    SlotBinding {
        slot,
        span: span(context, address, bytes),
        elem,
    }
}

fn valid() -> (
    Vec<SlotBinding>,
    Checked<Metadata>,
    Checked<Metadata>,
    MetadataRevision,
) {
    let revision = MetadataRevision::new();
    let bindings = vec![
        binding(0, 7, 0x1000, 8, ElemType::I32),
        binding(1, 7, 0x2000, 32, ElemType::I32),
        binding(2, 7, 0x3000, 32, ElemType::F32),
    ];
    // SAFETY: 测试区间代表相同宿主字节。
    let first = unsafe {
        Checked::<Metadata>::assume_uploaded(bindings[0].span, &[8, 7], 8, revision.clone())
            .unwrap()
    };
    // SAFETY: 测试区间代表相同宿主字节。
    let second = unsafe {
        Checked::<Metadata>::assume_uploaded(bindings[1].span, &[8, 7], 8, revision.clone())
            .unwrap()
    };
    (bindings, first, second, revision)
}

#[test]
fn accepts_complete_synchronous_plan() {
    let kernel = Kernel::<Example>::new().unwrap();
    let (bindings, first, second, _) = valid();
    let proofs = [first.binding(0), second.binding(1)];
    let plan = kernel.plan(&8, &bindings, &proofs).unwrap();
    assert_eq!(plan.dims().grid(), (1, 1, 1));
    assert_eq!(plan.dims().block(), (64, 1, 1));
}

#[test]
fn rejects_capacity_overlap_and_foreign_context() {
    let kernel = Kernel::<Example>::new().unwrap();
    let (bindings, first, second, _) = valid();
    let proofs = [first.binding(0), second.binding(1)];

    let mut short = bindings.clone();
    short[1].span = span(7, 0x2000, 28);
    assert!(matches!(
        kernel.plan(&8, &short, &proofs),
        Err(LauncherError::Capacity { slot: 1, .. })
    ));

    let mut overlap = bindings.clone();
    overlap[2].span = span(7, 0x2000, 32);
    assert_eq!(
        kernel.plan(&8, &overlap, &proofs).unwrap_err(),
        LauncherError::Alias {
            first: 1,
            second: 2,
        }
    );

    let mut foreign = bindings.clone();
    foreign[1].span = span(8, 0x2000, 32);
    assert_eq!(
        kernel.plan(&8, &foreign, &proofs).unwrap_err(),
        LauncherError::ContextMismatch { slot: 1 }
    );
}

#[test]
fn rejects_stale_or_mismatched_metadata() {
    let kernel = Kernel::<Example>::new().unwrap();
    let (bindings, first, second, revision) = valid();
    let proofs = [first.binding(0), second.binding(1)];

    assert!(matches!(
        kernel.plan(&7, &bindings, &proofs),
        Err(LauncherError::MetadataMismatch(_))
    ));

    revision.invalidate();
    assert_eq!(
        kernel.plan(&8, &bindings, &proofs).unwrap_err(),
        LauncherError::StaleMetadata
    );
}

#[test]
fn rejects_invalid_metadata_before_planning() {
    let revision = MetadataRevision::new();
    // SAFETY: 测试只检查宿主校验失败。
    assert!(matches!(
        unsafe { Checked::<Metadata>::assume_uploaded(span(7, 0x1000, 8), &[8, -1], 8, revision,) },
        Err(LauncherError::InvalidMetadata(_))
    ));
}

#[test]
fn requires_every_address_proof() {
    let kernel = Kernel::<Example>::new().unwrap();
    let (bindings, first, second, _) = valid();

    assert_eq!(
        kernel.plan(&8, &bindings, &[first.binding(0)]).unwrap_err(),
        LauncherError::MissingMetadataProof { slot: 1 }
    );
    assert_eq!(
        kernel
            .plan(
                &8,
                &bindings,
                &[first.binding(0), second.binding(1), second.binding(1)],
            )
            .unwrap_err(),
        LauncherError::DuplicateMetadataProof { slot: 1 }
    );
}
