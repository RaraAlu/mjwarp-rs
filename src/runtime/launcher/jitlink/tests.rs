use super::*;

fn elf(class: u8, machine: u16) -> Vec<u8> {
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"\x7fELF");
    bytes[4] = class;
    bytes[5] = 1;
    bytes[18..20].copy_from_slice(&machine.to_le_bytes());
    bytes
}

fn linker_error(result: Result<impl std::fmt::Debug, LauncherError>) -> String {
    match result {
        Err(LauncherError::Linker(detail)) => detail,
        other => panic!("期望链接错误，实际{other:?}"),
    }
}

#[test]
fn accepts_only_pinned_version() {
    assert!(check_version((12, 9)).is_ok());
    for version in [(12, 8), (12, 10), (13, 0), (11, 9)] {
        assert!(linker_error(check_version(version)).contains("版本不符"));
    }
}

#[test]
fn freezes_dlto_and_arch_options() {
    let options = link_options(89).unwrap();
    assert_eq!(options[0].to_str().unwrap(), "-dlto");
    assert_eq!(options[1].to_str().unwrap(), "-arch=sm_89");
    assert!(linker_error(link_options(0)).contains("架构"));
    assert!(linker_error(link_options(130)).contains("架构"));
}

#[test]
fn rejects_empty_oversized_and_misnamed_inputs() {
    let data = [1u8; 4];
    let input = |name, bytes| LinkInput {
        kind: LinkInputKind::Ltoir,
        name,
        bytes,
    };
    assert!(linker_error(validate_inputs(&[])).contains("没有链接输入"));
    assert!(linker_error(validate_inputs(&[input("a", &[])])).contains("输入为空"));
    assert!(linker_error(validate_inputs(&[input("", &data)])).contains("名称为空"));
    assert!(linker_error(validate_inputs(&[input("a\0b", &data)])).contains("NUL"));
    let many = vec![input("a", &data); MAX_LINK_INPUTS + 1];
    assert!(linker_error(validate_inputs(&many)).contains("过多"));
    let names = validate_inputs(&[input("a", &data), input("b", &data)]).unwrap();
    assert_eq!(names.len(), 2);
}

#[test]
fn symbols_carry_pinned_version_suffix() {
    assert_eq!(
        ffi::versioned("__nvJitLinkCreate"),
        "__nvJitLinkCreate_12_9"
    );
}

#[test]
fn input_kinds_follow_published_enum() {
    assert_eq!(LinkInputKind::Ltoir.raw(), 3);
    assert_eq!(LinkInputKind::Fatbin.raw(), 4);
}

#[test]
fn accepts_only_64bit_cuda_elf() {
    assert!(check_cubin(&elf(2, EM_CUDA)).is_ok());
    assert!(linker_error(check_cubin(&[])).contains("为空"));
    assert!(linker_error(check_cubin(b"\x7fELF")).contains("不是ELF"));
    assert!(linker_error(check_cubin(&[0u8; 64])).contains("不是ELF"));
    assert!(linker_error(check_cubin(&elf(1, EM_CUDA))).contains("64位"));
    // x86-64 机器号 62。
    assert!(linker_error(check_cubin(&elf(2, 62))).contains("机器号"));
}

#[test]
fn rejects_missing_and_relative_library_paths() {
    let missing = std::env::temp_dir().join("mjwarp-no-such-dir");
    assert!(linker_error(NvJitLink::load_from_dir(&missing)).contains("库缺失"));
    let relative = Path::new("nvJitLink_120_0.dll");
    assert!(linker_error(NvJitLink::load_file(relative)).contains("绝对路径"));
}

#[cfg(windows)]
#[test]
fn rejects_library_without_nvjitlink_symbols() {
    let root = std::env::var_os("SystemRoot").expect("SystemRoot");
    let kernel32 = PathBuf::from(root).join("System32").join("kernel32.dll");
    let detail = linker_error(NvJitLink::load_file(&kernel32));
    assert!(detail.contains("符号缺失"), "{detail}");
}
