//! W03：上下文内模块缓存。
//! 每份缓存只服务创建时的上下文实例。
//! 不同`CudaContext`实例一律视为不同上下文。
//! 同一产物只加载一次模块；各入口共用该模块。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use cudarc::driver::{CudaContext, CudaFunction, CudaModule};
use cudarc::nvrtc::Ptx;

use super::{Artifact, LauncherError};

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    // 锁内只改映射，不调用驱动；中毒时映射仍完整。
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// 已加载的入口；持有模块直至句柄释放。
#[derive(Clone, Debug)]
pub struct KernelHandle {
    function: CudaFunction,
    artifact: Arc<Artifact>,
    entry: &'static str,
}

impl KernelHandle {
    pub fn entry(&self) -> &'static str {
        self.entry
    }

    pub fn artifact(&self) -> &Arc<Artifact> {
        &self.artifact
    }

    /// 发射接口属于M3；此处只供运行时内部使用。
    #[allow(dead_code)]
    pub(crate) fn function(&self) -> &CudaFunction {
        &self.function
    }
}

/// 模块缓存计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModuleStats {
    pub module_loads: u64,
    pub function_loads: u64,
    pub hits: u64,
}

type ModuleSlot = Arc<OnceLock<Result<Arc<CudaModule>, LauncherError>>>;

/// 单个上下文的模块与函数缓存。
pub struct ModuleCache {
    context: Arc<CudaContext>,
    modules: Mutex<HashMap<String, ModuleSlot>>,
    functions: Mutex<HashMap<(String, &'static str), CudaFunction>>,
    module_loads: AtomicU64,
    function_loads: AtomicU64,
    hits: AtomicU64,
}

impl ModuleCache {
    pub fn new(context: Arc<CudaContext>) -> Self {
        Self {
            context,
            modules: Mutex::new(HashMap::new()),
            functions: Mutex::new(HashMap::new()),
            module_loads: AtomicU64::new(0),
            function_loads: AtomicU64::new(0),
            hits: AtomicU64::new(0),
        }
    }

    pub fn context(&self) -> &Arc<CudaContext> {
        &self.context
    }

    /// 取入口；未命中时加载模块或函数。
    pub fn function(
        &self,
        context: &Arc<CudaContext>,
        artifact: &Arc<Artifact>,
        entry: &'static str,
    ) -> Result<KernelHandle, LauncherError> {
        self.check_context(context)?;
        if let Some(handle) = self.lookup(artifact, entry) {
            return Ok(handle);
        }
        let module = self.module(artifact)?;
        self.context
            .bind_to_thread()
            .map_err(|e| LauncherError::Driver(format!("bind: {e:?}")))?;
        let function = module
            .load_function(entry)
            .map_err(|e| LauncherError::Driver(format!("{entry}: {e:?}")))?;
        self.function_loads.fetch_add(1, Ordering::Relaxed);
        let function = lock(&self.functions)
            .entry((artifact.key_sha256().to_owned(), entry))
            .or_insert(function)
            .clone();
        Ok(KernelHandle {
            function,
            artifact: Arc::clone(artifact),
            entry,
        })
    }

    /// 只查缓存，绝不调用驱动加载；供捕获期间使用。
    pub fn cached_function(
        &self,
        context: &Arc<CudaContext>,
        artifact: &Arc<Artifact>,
        entry: &'static str,
    ) -> Result<Option<KernelHandle>, LauncherError> {
        self.check_context(context)?;
        Ok(self.lookup(artifact, entry))
    }

    /// 移除该产物的模块与函数；已发出的句柄仍持有模块。
    pub fn invalidate(&self, artifact: &Artifact) -> bool {
        let key = artifact.key_sha256();
        lock(&self.functions).retain(|(k, _), _| k != key);
        lock(&self.modules).remove(key).is_some()
    }

    pub fn clear(&self) {
        lock(&self.functions).clear();
        lock(&self.modules).clear();
    }

    pub fn stats(&self) -> ModuleStats {
        ModuleStats {
            module_loads: self.module_loads.load(Ordering::Relaxed),
            function_loads: self.function_loads.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
        }
    }

    fn check_context(&self, context: &Arc<CudaContext>) -> Result<(), LauncherError> {
        if Arc::ptr_eq(&self.context, context) {
            Ok(())
        } else {
            Err(LauncherError::ModuleContext)
        }
    }

    fn lookup(&self, artifact: &Arc<Artifact>, entry: &'static str) -> Option<KernelHandle> {
        let function = lock(&self.functions)
            .get(&(artifact.key_sha256().to_owned(), entry))
            .cloned()?;
        self.hits.fetch_add(1, Ordering::Relaxed);
        Some(KernelHandle {
            function,
            artifact: Arc::clone(artifact),
            entry,
        })
    }

    fn module(&self, artifact: &Arc<Artifact>) -> Result<Arc<CudaModule>, LauncherError> {
        let key = artifact.key_sha256().to_owned();
        let slot = Arc::clone(lock(&self.modules).entry(key.clone()).or_default());
        let result = slot
            .get_or_init(|| {
                self.module_loads.fetch_add(1, Ordering::Relaxed);
                self.context
                    .load_module(Ptx::from_src(artifact.ptx().to_owned()))
                    .map_err(|e| LauncherError::Driver(format!("load_module: {e:?}")))
            })
            .clone();
        if result.is_err() {
            let mut modules = lock(&self.modules);
            if modules.get(&key).is_some_and(|s| Arc::ptr_eq(s, &slot)) {
                modules.remove(&key);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ArtifactCache, CompileRecipe, NvrtcSource, RecipePurpose};
    use super::*;
    use cudarc::driver::{LaunchConfig, PushKernelArg};

    const WARP32: LaunchConfig = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (32, 1, 1),
        shared_mem_bytes: 0,
    };
    const FILL: &str = r#"
extern "C" __global__ void fill_one(float* out) { out[threadIdx.x] = 1.0f; }
extern "C" __global__ void fill_two(float* out) { out[threadIdx.x] = 2.0f; }
"#;

    #[test]
    #[ignore = "requires an NVIDIA GPU and NVRTC"]
    fn gpu_entries_share_one_module_and_contexts_stay_isolated() {
        let artifacts = ArtifactCache::in_memory();
        let source = NvrtcSource::new("module_cache_fill", FILL).unwrap();
        let recipe = CompileRecipe::WARP_1_15_DIAGNOSTIC;
        let artifact = artifacts
            .compile(&source, &recipe, RecipePurpose::Diagnostic)
            .unwrap();
        let context = CudaContext::new(0).unwrap();
        let other = CudaContext::new(0).unwrap();
        let cache = ModuleCache::new(context.clone());
        let one = cache.function(&context, &artifact, "fill_one").unwrap();
        let two = cache.function(&context, &artifact, "fill_two").unwrap();
        let again = cache.function(&context, &artifact, "fill_one").unwrap();
        assert_eq!(
            cache.stats(),
            ModuleStats {
                module_loads: 1,
                function_loads: 2,
                hits: 1
            }
        );
        assert_eq!(again.entry(), "fill_one");
        assert!(matches!(
            cache.function(&other, &artifact, "fill_one"),
            Err(LauncherError::ModuleContext)
        ));
        assert!(matches!(
            cache.cached_function(&other, &artifact, "fill_one"),
            Err(LauncherError::ModuleContext)
        ));
        assert!(matches!(
            cache.function(&context, &artifact, "missing_entry"),
            Err(LauncherError::Driver(_))
        ));
        // 另一上下文实例另建缓存并独立加载。
        let isolated = ModuleCache::new(other.clone());
        isolated.function(&other, &artifact, "fill_one").unwrap();
        assert_eq!(isolated.stats().module_loads, 1);

        let stream = context.default_stream();
        let mut out = stream.alloc_zeros::<f32>(32).unwrap();
        for (handle, expected) in [(&one, 1.0f32), (&two, 2.0)] {
            let mut launch = stream.launch_builder(handle.function());
            launch.arg(&mut out);
            // SAFETY: Each kernel writes exactly blockDim.x=32 floats into `out`.
            unsafe { launch.launch(WARP32) }.unwrap();
            let values = stream.clone_dtoh(&out).unwrap();
            assert!(values.iter().all(|v| *v == expected));
        }

        // 失效后重新加载；旧句柄仍可用。
        assert!(cache.invalidate(&artifact));
        assert!(
            cache
                .cached_function(&context, &artifact, "fill_one")
                .unwrap()
                .is_none()
        );
        cache.function(&context, &artifact, "fill_one").unwrap();
        assert_eq!(cache.stats().module_loads, 2);
        let mut launch = stream.launch_builder(one.function());
        launch.arg(&mut out);
        // SAFETY: Same kernel and buffer contract as above.
        unsafe { launch.launch(WARP32) }.unwrap();
        stream.synchronize().unwrap();
    }
}
