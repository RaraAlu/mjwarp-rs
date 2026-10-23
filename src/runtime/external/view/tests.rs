//! A08@P10 零拷贝视图测试：指针相等（零拷贝证明）、与显式复制一致、所有权与
//! 跨流完成事件契约。

use super::*;

fn make_buffer() -> MirrorBuffer {
    MirrorBuffer::new(
        "state",
        0,
        0xAAAA,
        0x1000,
        7,
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
    )
}

fn completed(buffer: &MirrorBuffer) -> StreamFence {
    StreamFence::completed(buffer.stream, buffer.generation)
}

#[test]
fn view_is_zero_copy_and_matches_required_values() {
    let buffer = make_buffer();
    let layout = FieldLayout::contiguous(1, 3, 1);
    let view = ZeroCopyView::new(&buffer, layout, completed(&buffer)).unwrap();
    let slice = view.field().unwrap();
    assert_eq!(slice, &[2.0, 3.0, 4.0]);
    // 零拷贝证明：切片直接指向原缓冲（同一地址，无中间分配）。
    assert_eq!(
        slice.as_ptr() as usize,
        buffer.data().as_ptr() as usize + layout.offset * std::mem::size_of::<f32>()
    );
    assert!(std::ptr::eq(view.owner(), &buffer));
    assert_eq!(copy_field(&buffer, layout), slice);
}

#[test]
fn strided_view_yields_borrowed_slices() {
    let buffer = make_buffer();
    let layout = FieldLayout {
        offset: 0,
        width: 2,
        stride: 3,
        count: 2,
    };
    let view = ZeroCopyView::new(&buffer, layout, completed(&buffer)).unwrap();
    let elements: Vec<&[f32]> = view.elements().collect();
    assert_eq!(elements, vec![&[1.0, 2.0][..], &[4.0, 5.0][..]]);
    assert_eq!(elements[1].as_ptr(), unsafe {
        buffer.data().as_ptr().add(3)
    });
    assert_eq!(view.count(), 2);
}

#[test]
fn rejects_pending_event_and_cross_stream_events() {
    let buffer = make_buffer();
    let layout = FieldLayout::contiguous(0, 2, 1);
    let pending = StreamFence::pending(buffer.stream, buffer.generation);
    let error = ZeroCopyView::new(&buffer, layout, pending).unwrap_err();
    assert_eq!(
        error,
        ZeroCopyError::EventPending {
            stream: buffer.stream,
            generation: buffer.generation
        }
    );
    let other_stream = StreamFence::completed(buffer.stream + 1, buffer.generation);
    let error = ZeroCopyView::new(&buffer, layout, other_stream).unwrap_err();
    assert_eq!(
        error,
        ZeroCopyError::StreamMismatch {
            buffer_stream: buffer.stream,
            event_stream: buffer.stream + 1
        }
    );
    let stale = StreamFence::completed(buffer.stream, buffer.generation + 1);
    let error = ZeroCopyView::new(&buffer, layout, stale).unwrap_err();
    assert_eq!(
        error,
        ZeroCopyError::GenerationMismatch {
            buffer_generation: buffer.generation,
            event_generation: buffer.generation + 1
        }
    );
}

#[test]
fn write_updates_generation_and_event_completion_is_monotonic() {
    let mut buffer = make_buffer();
    let mut fence = StreamFence::pending(buffer.stream, 0);
    let generation = buffer.write(0, 9.0).unwrap();
    assert_eq!(generation, 1);
    assert_eq!(fence.generation, 0);
    fence.complete().unwrap();
    assert!(fence.is_completed());
    assert_eq!(
        fence.complete().unwrap_err(),
        ZeroCopyError::AlreadyCompleted {
            stream: buffer.stream
        }
    );
    // 写入后旧事件代数不匹配；新事件可建视图并看到新值。
    let error = ZeroCopyView::new(&buffer, FieldLayout::contiguous(0, 1, 1), fence).unwrap_err();
    assert!(matches!(error, ZeroCopyError::GenerationMismatch { .. }));
    let view = ZeroCopyView::new(
        &buffer,
        FieldLayout::contiguous(0, 1, 1),
        StreamFence::completed(buffer.stream, buffer.generation),
    )
    .unwrap();
    assert_eq!(view.field().unwrap(), &[9.0]);
}

#[test]
fn rejects_out_of_bounds_and_empty_buffers() {
    let buffer = make_buffer();
    let error = ZeroCopyView::new(
        &buffer,
        FieldLayout::contiguous(7, 2, 1),
        completed(&buffer),
    )
    .unwrap_err();
    assert!(matches!(error, ZeroCopyError::OutOfBounds { .. }));
    let error = ZeroCopyView::new(
        &buffer,
        FieldLayout::contiguous(0, 0, 1),
        completed(&buffer),
    )
    .unwrap_err();
    assert!(matches!(error, ZeroCopyError::OutOfBounds { .. }));
    let empty = MirrorBuffer::new("empty", 0, 0, 0, 7, Vec::new());
    let error = ZeroCopyView::new(
        &empty,
        FieldLayout::contiguous(0, 1, 1),
        StreamFence::completed(7, 0),
    )
    .unwrap_err();
    assert_eq!(error, ZeroCopyError::EmptyBuffer { name: "empty" });
    let mut buffer = make_buffer();
    assert!(matches!(
        buffer.write(99, 1.0).unwrap_err(),
        ZeroCopyError::OutOfBounds { .. }
    ));
}
