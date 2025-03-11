#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    AlreadySignaled,
    ConditionSatisfied,
    TimeOutExpired,
    WaitFailed,
}

impl Signal {
    pub fn signalled(self) -> bool {
        matches!(self, Self::AlreadySignaled | Self::ConditionSatisfied)
    }
}

impl TryFrom<gl::types::GLenum> for Signal {
    type Error = String;

    fn try_from(value: gl::types::GLenum) -> Result<Self, Self::Error> {
        match value {
            gl::ALREADY_SIGNALED => Ok(Self::AlreadySignaled),
            gl::CONDITION_SATISFIED => Ok(Self::ConditionSatisfied),
            gl::TIMEOUT_EXPIRED => Ok(Self::TimeOutExpired),
            gl::WAIT_FAILED => Ok(Self::WaitFailed),
            _ => Err(format!("Invalid Enum: {}", value)),
        }
    }
}

#[derive(Debug)]
struct SyncObject {
    sync: gl::types::GLsync,
}

impl SyncObject {
    pub fn new() -> Self {
        let sync = unsafe { gl::FenceSync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0) };
        Self { sync }
    }

    pub fn signalled(&self) -> bool {
        // Could use gl::GetSynciv with gl::SYNC_STATUS to check if the fence is signalled, but
        // gl::ClientWaitSync seems to be faster on NVIDIA GPUs
        self.block_cpu(0).signalled()
    }

    pub fn block_cpu(&self, timeout_ns: usize) -> Signal {
        let signal: Signal = unsafe {
            gl::ClientWaitSync(
                self.sync,
                gl::SYNC_FLUSH_COMMANDS_BIT,
                timeout_ns as gl::types::GLuint64,
            )
        }
        .try_into()
        .expect("Failed to convert GLenum to Signal when blocking CPU");

        signal
    }

    #[allow(dead_code)]
    pub fn wait_gpu(&self) {
        unsafe { gl::WaitSync(self.sync, 0, gl::TIMEOUT_IGNORED) }
    }
}

impl Drop for SyncObject {
    fn drop(&mut self) {
        unsafe { gl::DeleteSync(self.sync) }
    }
}

#[derive(Debug, Default)]
pub struct Fence {
    fence: Option<SyncObject>,
}

impl Fence {
    pub fn start(&mut self) -> &mut Self {
        self.fence = Some(SyncObject::new());
        self
    }

    pub fn signalled(&self) -> bool {
        self.fence.as_ref().is_none_or(|fence| fence.signalled())
    }
}
