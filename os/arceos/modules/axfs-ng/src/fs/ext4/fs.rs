use alloc::sync::Arc;
use core::cell::OnceCell;

use axdriver::{AxBlockDevice, prelude::BlockDriverOps};
use axfs_ng_vfs::{
    DirEntry, DirNode, Filesystem, FilesystemOps, Reference, StatFs, VfsResult, path::MAX_NAME_LEN,
};
use kspin::{SpinNoPreempt as Mutex, SpinNoPreemptGuard as MutexGuard};
use lwext4_rust::{FsConfig, ffi::EXT4_ROOT_INO};

use super::{
    Ext4Disk, Inode,
    util::{LwExt4Filesystem, into_vfs_err},
};

const EXT4_CONFIG: FsConfig = FsConfig { bcache_size: 256 };

pub struct Ext4Filesystem {
    inner: Mutex<LwExt4Filesystem>,
    root_dir: OnceCell<DirEntry>,
}

impl Ext4Filesystem {
    pub fn new(dev: AxBlockDevice) -> VfsResult<Filesystem> {
        let mut dev = dev;
        log_ext4_probe_info(&mut dev);

        let ext4 = match lwext4_rust::Ext4Filesystem::new(Ext4Disk(dev), EXT4_CONFIG) {
            Ok(ext4) => ext4,
            Err(err) => {
                error!("failed to mount ext4 rootfs: {err:?}");
                return Err(into_vfs_err(err));
            }
        };

        let fs = Arc::new(Self {
            inner: Mutex::new(ext4),
            root_dir: OnceCell::new(),
        });
        let _ = fs.root_dir.set(DirEntry::new_dir(
            |this| DirNode::new(Inode::new(fs.clone(), EXT4_ROOT_INO, Some(this))),
            Reference::root(),
        ));
        Ok(Filesystem::new(fs))
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, LwExt4Filesystem> {
        self.inner.lock()
    }
}

unsafe impl Send for Ext4Filesystem {}

unsafe impl Sync for Ext4Filesystem {}

impl FilesystemOps for Ext4Filesystem {
    fn name(&self) -> &str {
        "ext4"
    }

    fn root_dir(&self) -> DirEntry {
        self.root_dir.get().unwrap().clone()
    }

    fn stat(&self) -> VfsResult<StatFs> {
        let mut fs = self.lock();
        let stat = fs.stat().map_err(into_vfs_err)?;
        Ok(StatFs {
            fs_type: 0xef53,
            block_size: stat.block_size as _,
            blocks: stat.blocks_count,
            blocks_free: stat.free_blocks_count,
            blocks_available: stat.free_blocks_count,

            file_count: stat.inodes_count as _,
            free_file_count: stat.free_inodes_count as _,

            name_length: MAX_NAME_LEN as _,
            fragment_size: 0,
            mount_flags: 0,
        })
    }

    fn flush(&self) -> VfsResult<()> {
        self.inner.lock().flush().map_err(into_vfs_err)
    }
}

fn log_ext4_probe_info(dev: &mut AxBlockDevice) {
    // info!(
    //     "probing ext4 on block device {:?}: blocks={}, block_size={}",
    //     dev.device_name(),
    //     dev.num_blocks(),
    //     dev.block_size()
    // );

    let mut block0 = [0u8; 4096];
    if let Err(err) = dev.read_block(0, &mut block0) {
        warn!("failed to read ext4 probe block 0: {err:?}");
        return;
    }

    let sb = &block0[1024..1024 + 1024];
    let magic = u16::from_le_bytes([sb[0x38], sb[0x39]]);
    let log_block_size = u32::from_le_bytes([sb[0x18], sb[0x19], sb[0x1a], sb[0x1b]]);
    let block_size = 1024u32 << log_block_size;
    let compat = u32::from_le_bytes([sb[0x5c], sb[0x5d], sb[0x5e], sb[0x5f]]);
    let incompat = u32::from_le_bytes([sb[0x60], sb[0x61], sb[0x62], sb[0x63]]);
    let ro_compat = u32::from_le_bytes([sb[0x64], sb[0x65], sb[0x66], sb[0x67]]);

    info!(
        "ext4 superblock: magic={:#x}, block_size={}, compat={:#x}, incompat={:#x}, ro_compat={:#x}",
        magic, block_size, compat, incompat, ro_compat
    );
}
