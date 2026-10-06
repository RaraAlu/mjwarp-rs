//! 固定样本只提供探针参考。
//! 宿主参考不替代GPU执行。

use super::{BLOCK_THREADS, GUARD_ELEMENTS, GUARD_VALUE, ProbeKernel, input_values};
use crate::diagnostics::ProbeError;

const INTEGER_GUARD: u32 = 0xdead_beef;
const SOLVE_TOLERANCE: f64 = 1.0e-6;

pub(super) struct Samples<T> {
    pub input: Vec<T>,
    pub initial: Vec<T>,
    pub expected: Vec<T>,
}

pub(super) fn integers(kernel: ProbeKernel, elements: usize, epoch: usize) -> Samples<u32> {
    let input: Vec<u32> = (0..elements)
        .map(|index| 1 + (index % 17) as u32 + epoch as u32)
        .collect();
    let mut expected = match kernel {
        ProbeKernel::AtomicSum => vec![input.iter().sum()],
        ProbeKernel::BlockReduce => input
            .chunks(BLOCK_THREADS)
            .map(|block| block.iter().sum())
            .collect(),
        ProbeKernel::BlockScan => input
            .chunks(BLOCK_THREADS)
            .flat_map(|block| {
                block.iter().scan(0, |sum, value| {
                    *sum += value;
                    Some(*sum)
                })
            })
            .collect(),
        ProbeKernel::ControlFlow => input
            .iter()
            .map(|&value| {
                // 闭式参考不复制设备循环。
                let iterations = value % 7 + 1;
                let triangle = iterations * (iterations - 1) / 2;
                if value.is_multiple_of(2) {
                    iterations * value + triangle
                } else {
                    2 * iterations * value - triangle
                }
            })
            .collect(),
        _ => unreachable!("浮点内核不使用整数样本"),
    };
    let mut initial = vec![INTEGER_GUARD; expected.len()];
    if kernel == ProbeKernel::AtomicSum {
        // 累加器每轮先清零。
        initial.fill(0);
    }
    initial.extend([INTEGER_GUARD; GUARD_ELEMENTS]);
    expected.extend([INTEGER_GUARD; GUARD_ELEMENTS]);
    Samples {
        input,
        initial,
        expected,
    }
}

pub(super) fn floats(kernel: ProbeKernel, elements: usize, epoch: usize) -> Samples<f32> {
    let (input, mut expected) = match kernel {
        ProbeKernel::Affine => {
            let input = input_values(elements, epoch);
            let expected = input.iter().map(|value| value * 2.0 + 1.0).collect();
            (input, expected)
        }
        ProbeKernel::SmallSolve => {
            // 已知解生成右端，避免复制求解算法。
            // A固定为[[4,1],[1,2]]。
            let mut input = Vec::with_capacity(elements * 2);
            let mut expected = Vec::with_capacity(elements * 2);
            for index in 0..elements {
                let x = ((index % 9) as f32 - 4.0) * 0.25 + epoch as f32 * 0.5;
                let y = ((index % 7) as f32 - 3.0) * 0.5 - epoch as f32 * 0.25;
                input.extend([4.0 * x + y, x + 2.0 * y]);
                expected.extend([x, y]);
            }
            (input, expected)
        }
        _ => unreachable!("整数内核不使用浮点样本"),
    };
    let initial = vec![GUARD_VALUE; expected.len() + GUARD_ELEMENTS];
    expected.extend([GUARD_VALUE; GUARD_ELEMENTS]);
    Samples {
        input,
        initial,
        expected,
    }
}

fn validate_length(expected: usize, actual: usize) -> Result<(), ProbeError> {
    if actual != expected {
        return Err(ProbeError::InvalidOutputLength { expected, actual });
    }
    Ok(())
}

pub(super) fn validate_integers(
    _kernel: ProbeKernel,
    samples: &Samples<u32>,
    actual: &[u32],
) -> Result<(), ProbeError> {
    validate_length(samples.expected.len(), actual.len())?;
    for (index, (&expected, &actual)) in samples.expected.iter().zip(actual).enumerate() {
        if expected != actual {
            return Err(ProbeError::IntegerMismatch {
                index,
                expected,
                actual,
            });
        }
    }
    Ok(())
}

pub(super) fn validate_floats(
    kernel: ProbeKernel,
    samples: &Samples<f32>,
    actual: &[f32],
) -> Result<(), ProbeError> {
    if kernel == ProbeKernel::Affine {
        return super::validate_output(&samples.input, actual);
    }
    validate_length(samples.expected.len(), actual.len())?;
    let active = actual.len() - GUARD_ELEMENTS;
    for (index, (&expected, &actual)) in samples.expected.iter().zip(actual).enumerate() {
        if kernel == ProbeKernel::SmallSolve && index < active {
            close(index, expected.into(), actual.into())?;
        } else if expected != actual {
            return Err(ProbeError::Mismatch {
                index,
                expected,
                actual,
            });
        }
    }
    if kernel == ProbeKernel::SmallSolve {
        // 用f64核对两行残差。
        for (system, (solution, rhs)) in actual[..active]
            .as_chunks::<2>()
            .0
            .iter()
            .zip(samples.input.as_chunks::<2>().0)
            .enumerate()
        {
            let x = f64::from(solution[0]);
            let y = f64::from(solution[1]);
            close(system * 2, rhs[0].into(), 4.0 * x + y)?;
            close(system * 2 + 1, rhs[1].into(), x + 2.0 * y)?;
        }
    }
    Ok(())
}

fn close(index: usize, expected: f64, actual: f64) -> Result<(), ProbeError> {
    if !actual.is_finite() || (actual - expected).abs() > SOLVE_TOLERANCE * (1.0 + expected.abs()) {
        return Err(ProbeError::SolveMismatch {
            index,
            expected,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixes_integer_references_across_partial_blocks() {
        let reduction = integers(ProbeKernel::BlockReduce, 129, 0);
        assert_eq!(reduction.expected[..2], [1116, 10]);
        let scan = integers(ProbeKernel::BlockScan, 129, 0);
        assert_eq!(scan.expected[0], 1);
        assert_eq!(scan.expected[127], 1116);
        assert_eq!(scan.expected[128], 10);
        let atomic = integers(ProbeKernel::AtomicSum, 129, 0);
        assert_eq!(atomic.expected[0], 1126);
        assert_eq!(atomic.initial[0], 0);
        let control = integers(ProbeKernel::ControlFlow, 4, 0);
        assert_eq!(control.expected[..4], [3, 9, 18, 30]);
    }

    #[test]
    fn checks_all_reference_values_and_guards() {
        for kernel in ProbeKernel::ALL {
            if matches!(kernel, ProbeKernel::Affine | ProbeKernel::SmallSolve) {
                let samples = floats(kernel, 129, 3);
                validate_floats(kernel, &samples, &samples.expected).unwrap();
            } else {
                let samples = integers(kernel, 129, 3);
                validate_integers(kernel, &samples, &samples.expected).unwrap();
            }
        }
    }

    #[test]
    fn rejects_stale_integer_results_and_guard_changes() {
        for kernel in [
            ProbeKernel::AtomicSum,
            ProbeKernel::BlockReduce,
            ProbeKernel::BlockScan,
            ProbeKernel::ControlFlow,
        ] {
            let old = integers(kernel, 129, 0);
            let current = integers(kernel, 129, 1);
            assert!(validate_integers(kernel, &current, &old.expected).is_err());
            assert!(validate_integers(kernel, &current, &[]).is_err());
            let mut output = current.expected.clone();
            *output.last_mut().unwrap() = 0;
            assert!(validate_integers(kernel, &current, &output).is_err());
        }
    }

    #[test]
    fn rejects_wrong_solutions_nonfinite_values_and_guards() {
        let kernel = ProbeKernel::SmallSolve;
        let samples = floats(kernel, 129, 3);
        for value in [0.0, f32::NAN, f32::INFINITY] {
            let mut output = samples.expected.clone();
            output[0] = value;
            assert!(validate_floats(kernel, &samples, &output).is_err());
        }
        let mut output = samples.expected.clone();
        *output.last_mut().unwrap() += 1.0;
        assert!(validate_floats(kernel, &samples, &output).is_err());
        assert!(validate_floats(kernel, &samples, &[]).is_err());
        assert!(validate_floats(kernel, &samples, &floats(kernel, 129, 2).expected).is_err());
    }

    #[test]
    fn validates_known_solutions_and_independent_residuals() {
        let samples = floats(ProbeKernel::SmallSolve, 1, 0);
        assert_eq!(samples.input, [-5.5, -4.0]);
        assert_eq!(samples.expected[..2], [-1.0, -1.5]);
        let mut bad_rhs = floats(ProbeKernel::SmallSolve, 1, 0);
        bad_rhs.input[0] += 0.01;
        assert!(validate_floats(ProbeKernel::SmallSolve, &bad_rhs, &bad_rhs.expected).is_err());
    }

    #[test]
    fn keeps_integer_sum_in_range_at_replay_limit() {
        let samples = integers(
            ProbeKernel::AtomicSum,
            crate::runtime::MAX_ELEMENTS,
            crate::runtime::MAX_REPLAYS,
        );
        let wide_sum: u64 = samples.input.iter().map(|&value| u64::from(value)).sum();
        assert!(wide_sum <= u64::from(u32::MAX));
        assert_eq!(u64::from(samples.expected[0]), wide_sum);
        assert_eq!(samples.initial[0], 0);
    }

    #[test]
    fn freezes_solve_tolerance_boundary() {
        close(0, 0.0, SOLVE_TOLERANCE).unwrap();
        assert!(close(0, 0.0, SOLVE_TOLERANCE * 1.01).is_err());
        assert!(close(0, 0.0, f64::NAN).is_err());
    }
}
