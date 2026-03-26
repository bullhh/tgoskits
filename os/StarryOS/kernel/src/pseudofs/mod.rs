//! Basic virtual filesystem support

pub mod dev;
mod device;
mod dir;
mod file;
mod fs;
mod proc;
mod tmp;

use alloc::sync::Arc;

use axerrno::LinuxResult;
use axfs::{FS_CONTEXT, FsContext, ROOT_FS_CONTEXT};

/// 调试函数：读取 TPIDR_EL1 寄存器 (percpu crate 使用 EL1)
#[cfg(target_arch = "aarch64")]
fn read_tpidr_el1() -> usize {
    let val: usize;
    unsafe {
        core::arch::asm!("mrs {}, tpidr_el1", out(reg) val);
    }
    val
}

#[cfg(not(target_arch = "aarch64"))]
fn read_tpidr_el1() -> usize {
    0
}
use axfs_ng_vfs::{
    DirNodeOps, FileNodeOps, Filesystem, NodePermission, WeakDirEntry,
    path::{Path, PathBuf},
};
pub use tmp::MemoryFs;

pub use self::{device::*, dir::*, file::*, fs::*};

/// A callback that builds a `Arc<dyn DirNodeOps>` for a given
/// `WeakDirEntry`.
pub type DirMaker = Arc<dyn Fn(WeakDirEntry) -> Arc<dyn DirNodeOps> + Send + Sync>;

/// An enum containing either a directory ([`DirMaker`]) or a file (`Arc<dyn
/// FileNodeOps>`).
#[derive(Clone)]
pub enum NodeOpsMux {
    /// A directory node.
    Dir(DirMaker),
    /// A file node.
    File(Arc<dyn FileNodeOps>),
}

impl From<DirMaker> for NodeOpsMux {
    fn from(maker: DirMaker) -> Self {
        Self::Dir(maker)
    }
}

impl<T: FileNodeOps> From<Arc<T>> for NodeOpsMux {
    fn from(ops: Arc<T>) -> Self {
        Self::File(ops)
    }
}

const DIR_PERMISSION: NodePermission = NodePermission::from_bits_truncate(0o755);

fn mount_at(fs: &FsContext, path: &str, mount_fs: Filesystem) -> LinuxResult<()> {
    if fs.resolve(path).is_err() {
        fs.create_dir(path, DIR_PERMISSION)?;
    }
    fs.resolve(path)?.mount(&mount_fs)?;
    info!("Mounted {} at {}", mount_fs.name(), path);
    Ok(())
}

/// Mount all filesystems
pub fn mount_all() -> LinuxResult<()> {
    info!("Initialize pseudofs...");
    info!("ROOT_FS_CONTEXT initialized: {}", ROOT_FS_CONTEXT.get().is_some());
    
    // === 调试信息：检查 TPIDR_EL1 (percpu crate 使用 EL1) ===
    let tpidr = read_tpidr_el1();
    info!("TPIDR_EL1 before FS_CONTEXT.lock: {:#x}", tpidr);
    
    // === 调试信息：检查是否启用了 multitask ===
    info!("multitask feature: {}", cfg!(feature = "multitask"));
    
    // === 调试信息：尝试获取锁 ===
    info!("About to lock FS_CONTEXT...");
    let fs = FS_CONTEXT.lock();
    info!("FS_CONTEXT lock acquired successfully");
    mount_at(&fs, "/dev", dev::new_devfs())?;
    mount_at(&fs, "/dev/shm", tmp::MemoryFs::new())?;
    mount_at(&fs, "/tmp", tmp::MemoryFs::new())?;
    mount_at(&fs, "/proc", proc::new_procfs())?;

    mount_at(&fs, "/sys", tmp::MemoryFs::new())?;
    info!("Mounted sysfs at /sys OK");
    let mut path = PathBuf::new();
    for comp in Path::new("/sys/class/graphics/fb0/device").components() {
        path.push(comp.as_str());
        if fs.resolve(&path).is_err() {
            fs.create_dir(&path, DIR_PERMISSION)?;
        }
    }
    path.push("subsystem");
    fs.symlink("whatever", &path)?;
    drop(fs);

    #[cfg(feature = "dev-log")]
    dev::bind_dev_log().expect("Failed to bind /dev/log");

    Ok(())
}
