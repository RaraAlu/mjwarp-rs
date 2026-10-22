//! M5：异步提交凭证与访问依赖账本。
//! W06：凭证持有资源直到完成。
//! W07：延迟错误归属与依赖隔离。
//! W08：待回收列表；完成查询不释放资源。
//! W20：同一设备上的读写顺序按跨流依赖账本保证。
//! 同步接口保持不变；异步入口只改变等待位置。

use super::LauncherError;
use std::ops::Range;

/// 完成栅栏：只查询与等待，不持有资源所有权。
pub trait Fence {
    fn query(&self) -> Result<bool, LauncherError>;
    fn wait(&self) -> Result<(), LauncherError>;
}

/// 提交凭证持有的资源。
/// 隔离失败资源，避免后续复用。
pub trait RetainedResources {
    fn quarantine(&self);
}

impl RetainedResources for () {
    fn quarantine(&self) {}
}

impl<T: RetainedResources> RetainedResources for Vec<T> {
    fn quarantine(&self) {
        for resource in self {
            resource.quarantine();
        }
    }
}

/// 设备、上下文与流的身份。
/// 同设备不同流属于不同提交域。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamKey {
    device: usize,
    context: usize,
    stream: u64,
}

impl StreamKey {
    pub fn new(device: usize, context: usize, stream: u64) -> Self {
        Self {
            device,
            context,
            stream,
        }
    }

    pub fn device(&self) -> usize {
        self.device
    }

    pub fn context(&self) -> usize {
        self.context
    }

    pub fn stream(&self) -> u64 {
        self.stream
    }
}

/// 缓冲访问权限。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode {
    Read,
    Write,
}

/// 一次提交声明的设备字节区间访问。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferAccess {
    device: usize,
    context: usize,
    range: Range<u64>,
    mode: AccessMode,
}

impl BufferAccess {
    pub fn read(device: usize, context: usize, range: Range<u64>) -> Result<Self, LauncherError> {
        Self::new(device, context, range, AccessMode::Read)
    }

    pub fn write(device: usize, context: usize, range: Range<u64>) -> Result<Self, LauncherError> {
        Self::new(device, context, range, AccessMode::Write)
    }

    fn new(
        device: usize,
        context: usize,
        range: Range<u64>,
        mode: AccessMode,
    ) -> Result<Self, LauncherError> {
        if range.start >= range.end {
            return Err(LauncherError::InvalidSpan("提交访问区间为空或倒置"));
        }
        Ok(Self {
            device,
            context,
            range,
            mode,
        })
    }

    pub fn range(&self) -> Range<u64> {
        self.range.clone()
    }

    pub fn mode(&self) -> AccessMode {
        self.mode
    }

    /// 同设备同上下文且区间重叠，并且至少一侧写。
    fn conflicts(&self, other: &Self) -> bool {
        self.device == other.device
            && self.context == other.context
            && self.range.start < other.range.end
            && other.range.start < self.range.end
            && (self.mode == AccessMode::Write || other.mode == AccessMode::Write)
    }
}

/// 提交编号。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubmissionId(u64);

impl SubmissionId {
    /// 以持久化数值重建提交编号。
    pub fn from_value(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// 提交状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionState {
    Pending,
    Completed,
    Failed,
}

struct Entry<F: Fence, R: RetainedResources> {
    id: u64,
    stream: StreamKey,
    accesses: Vec<BufferAccess>,
    dependencies: Vec<u64>,
    fence: F,
    resources: R,
    state: SubmissionState,
    error: Option<LauncherError>,
}

impl<F: Fence, R: RetainedResources> Entry<F, R> {
    fn new(
        id: u64,
        stream: StreamKey,
        accesses: Vec<BufferAccess>,
        dependencies: Vec<u64>,
        resources: R,
        fence: F,
    ) -> Self {
        Self {
            id,
            stream,
            accesses,
            dependencies,
            fence,
            resources,
            state: SubmissionState::Pending,
            error: None,
        }
    }

    fn mark_failed(&mut self, error: LauncherError) {
        self.resources.quarantine();
        if self.state != SubmissionState::Failed {
            self.state = SubmissionState::Failed;
            self.error = Some(error);
        }
    }
}

/// 异步提交队列。
///
/// 调用方必须先登记提交，再启动GPU工作。
/// 凭证持有资源，直到成功等待后才释放。
/// 依赖必须是本队列中尚未完成的提交。
pub struct SubmitQueue<F: Fence, R: RetainedResources> {
    next_id: u64,
    entries: Vec<Entry<F, R>>,
}

impl<F: Fence, R: RetainedResources> Default for SubmitQueue<F, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: Fence, R: RetainedResources> SubmitQueue<F, R> {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            entries: Vec::new(),
        }
    }

    /// 登记一次提交：先校验依赖与访问账本，再保留资源。
    /// 任一校验失败都不保留资源，也不发射任务。
    pub fn submit(
        &mut self,
        stream: StreamKey,
        accesses: &[BufferAccess],
        dependencies: &[SubmissionId],
        resources: R,
        fence: F,
    ) -> Result<SubmissionId, LauncherError> {
        self.check_dependencies(dependencies)?;
        self.check_accesses(stream, accesses, dependencies)?;
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or(LauncherError::Overflow("submission_id"))?;
        let entry = Entry::new(
            id,
            stream,
            accesses.to_vec(),
            dependencies.iter().map(|id| id.value()).collect(),
            resources,
            fence,
        );
        self.entries.push(entry);
        Ok(SubmissionId(id))
    }

    /// 查询完成：已完成返回真，未完成返回假。
    /// 查询完成不释放资源。
    pub fn query(&mut self, id: SubmissionId) -> Result<bool, LauncherError> {
        match self.state_of(id)? {
            SubmissionState::Completed => return Ok(true),
            SubmissionState::Failed => return Err(self.stored_error(id)?),
            SubmissionState::Pending => {}
        }
        let dependencies = self.entry(id)?.dependencies.clone();
        for dependency in dependencies {
            let dependency = SubmissionId(dependency);
            match self.query(dependency) {
                Ok(true) => {}
                Ok(false) => return Ok(false),
                Err(_) => {
                    let fault = LauncherError::FaultedUpstream(dependency.value());
                    self.fail_entry(id, fault.clone());
                    return Err(fault);
                }
            }
        }
        let ready = match self.entry(id)?.fence.query() {
            Ok(ready) => ready,
            Err(error) => {
                self.fail_entry(id, error.clone());
                return Err(error);
            }
        };
        if ready && let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id.0) {
            entry.state = SubmissionState::Completed;
            entry.error = None;
        }
        Ok(ready)
    }

    /// 等待完成并释放资源。
    /// 依赖失败时本提交按上游失败归属，且不等待无关提交。
    pub fn wait(&mut self, id: SubmissionId) -> Result<R, LauncherError> {
        match self.state_of(id)? {
            SubmissionState::Completed => return self.retire(id),
            SubmissionState::Failed => return Err(self.stored_error(id)?),
            SubmissionState::Pending => {}
        }
        let dependencies = self.entry(id)?.dependencies.clone();
        for dependency in dependencies {
            let dependency = SubmissionId(dependency);
            if self.wait(dependency).is_err() {
                let fault = LauncherError::FaultedUpstream(dependency.value());
                self.fail_entry(id, fault.clone());
                return Err(fault);
            }
        }
        match self.state_of(id)? {
            SubmissionState::Completed => return self.retire(id),
            SubmissionState::Failed => return Err(self.stored_error(id)?),
            SubmissionState::Pending => {}
        }
        let waited = {
            let entry = self.entry(id)?;
            entry.fence.wait()
        };
        if let Err(error) = waited {
            self.fail_entry(id, error.clone());
            return Err(error);
        }
        let resources = self.retire(id)?;
        Ok(resources)
    }

    /// 提交在发射后失败：保留资源并记录归属错误。
    /// 错误不会被后续成功的查询覆盖。
    pub fn fail(&mut self, id: SubmissionId, error: LauncherError) -> Result<(), LauncherError> {
        self.fail_entry(id, error);
        Ok(())
    }

    pub fn state(&self, id: SubmissionId) -> Result<SubmissionState, LauncherError> {
        self.state_of(id)
    }

    /// 尚未回收的提交数。
    pub fn outstanding(&self) -> usize {
        self.entries.len()
    }

    fn entry(&self, id: SubmissionId) -> Result<&Entry<F, R>, LauncherError> {
        self.entries
            .iter()
            .find(|entry| entry.id == id.0)
            .ok_or(LauncherError::UnknownSubmission(id.0))
    }

    fn state_of(&self, id: SubmissionId) -> Result<SubmissionState, LauncherError> {
        Ok(self.entry(id)?.state)
    }

    fn stored_error(&self, id: SubmissionId) -> Result<LauncherError, LauncherError> {
        let entry = self.entry(id)?;
        Ok(match &entry.error {
            Some(error) => error.clone(),
            None => LauncherError::FaultedUpstream(id.0),
        })
    }

    fn fail_entry(&mut self, id: SubmissionId, error: LauncherError) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id.0) {
            entry.mark_failed(error);
        }
    }

    /// 回收已完成或失败的提交，返回其资源。
    fn retire(&mut self, id: SubmissionId) -> Result<R, LauncherError> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.id == id.0)
            .ok_or(LauncherError::UnknownSubmission(id.0))?;
        if self.entries[index].state == SubmissionState::Failed {
            return Err(self.stored_error(id)?);
        }
        Ok(self.entries.remove(index).resources)
    }

    fn check_dependencies(&self, dependencies: &[SubmissionId]) -> Result<(), LauncherError> {
        for dependency in dependencies {
            self.entry(*dependency)?;
        }
        for (position, dependency) in dependencies.iter().enumerate() {
            if dependencies[..position].contains(dependency) {
                return Err(LauncherError::DuplicateDependency(dependency.0));
            }
            match self.entry(*dependency) {
                Ok(entry) if entry.state == SubmissionState::Failed => {
                    return Err(LauncherError::FaultedUpstream(dependency.0));
                }
                Ok(entry) if entry.state == SubmissionState::Completed => {
                    // 已完成提交不再要求依赖；资源仍由账本持有至回收。
                }
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    /// 先按同流顺序放行；跨流重叠访问必须声明依赖。
    fn check_accesses(
        &self,
        stream: StreamKey,
        accesses: &[BufferAccess],
        dependencies: &[SubmissionId],
    ) -> Result<(), LauncherError> {
        let dependency_values: Vec<u64> = dependencies.iter().map(|id| id.value()).collect();
        for entry in &self.entries {
            if entry.state != SubmissionState::Pending {
                continue;
            }
            if entry.stream == stream {
                continue;
            }
            if !entry
                .accesses
                .iter()
                .any(|previous| accesses.iter().any(|access| access.conflicts(previous)))
            {
                continue;
            }
            if !self.dependency_reaches(&dependency_values, entry.id) {
                return Err(LauncherError::AccessConflict { blocking: entry.id });
            }
        }
        Ok(())
    }

    /// 依赖闭包可达性；只沿未回收提交的边。
    fn dependency_reaches(&self, dependencies: &[u64], target: u64) -> bool {
        let mut stack = dependencies.to_vec();
        let mut visited: Vec<u64> = Vec::new();
        while let Some(id) = stack.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.push(id);
            if id == target {
                return true;
            }
            if let Some(entry) = self.entries.iter().find(|entry| entry.id == id) {
                stack.extend(entry.dependencies.iter().copied());
            }
        }
        false
    }
}

impl<F: Fence, R: RetainedResources> Drop for SubmitQueue<F, R> {
    fn drop(&mut self) {
        for entry in self.entries.drain(..) {
            let waited =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| entry.fence.wait()));
            match waited {
                Ok(Ok(())) => {}
                _ => {
                    entry.resources.quarantine();
                    // 无法证明完成时保守保留资源与栅栏。
                    std::mem::forget(entry);
                }
            }
        }
    }
}
