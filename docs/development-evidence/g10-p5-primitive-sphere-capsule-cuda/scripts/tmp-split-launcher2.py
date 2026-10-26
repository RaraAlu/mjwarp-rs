import io, re

root = r'D:\Rust\mjwarp-rs'
main_path = root + r'\tests\g10_p5_nvrtc_launcher.rs'
support_path = root + r'\tests\support\launcher_compare_support.rs'

support = io.open(support_path, encoding='utf-8').read()
proof_start = support.index('#[test]\nfn world_order_normalization_turns_row_swap_into_zero_diff()')
proof = support[proof_start:]
support = support[:proof_start].rstrip('\n') + '\n'

# make cross-module items visible to the parent test module
for name in (
    'fn check_exact<',
    'fn check_exact_f32(',
    'fn check_dist(',
    'fn check_vectors(',
    'fn g5_contact_order(',
    'fn permute_rows_f32(',
    'fn permute_rows_i32(',
):
    support = support.replace('\\n' + name, '\\npub(crate) ' + name, 1)
    support = support.replace('\n' + name, '\npub(crate) ' + name, 1)
support = re.sub(r'\nfn print\(', '\npub(crate) fn print(', support, 1)
support = support.replace('\nfn print(&self', '\npub(crate) fn print(&self', 1)
io.open(support_path, 'w', encoding='utf-8', newline='\n').write(support)

main = io.open(main_path, encoding='utf-8').read()
main = main.replace(
    'use launcher_compare::compare_case;',
    'use launcher_compare::{\n    check_dist, check_exact, check_exact_f32, check_vectors, compare_case,\n    g5_contact_order, permute_rows_f32, permute_rows_i32, CaseReport,\n};',
)
main = main.rstrip('\n') + '\n\n' + proof
io.open(main_path, 'w', encoding='utf-8', newline='\n').write(main)
print('proof moved to main; support lines preserved')
