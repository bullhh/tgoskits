use core::ops::Range;

use axdriver_base::{BaseDriverOps, DevError, DevResult, DeviceType};
use gpt_disk_io::{
    BlockIo, Disk, DiskError,
    gpt_disk_types::{BlockSize, GptPartitionEntry, Lba},
};
use log::{debug, info};

use crate::BlockDriverOps;

struct BlockDriverAdapter<'a, T>(&'a mut T);

impl<T: BlockDriverOps> BlockIo for BlockDriverAdapter<'_, T> {
    type Error = DevError;

    fn block_size(&self) -> BlockSize {
        BlockSize::from_usize(self.0.block_size()).unwrap()
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.num_blocks())
    }

    fn read_blocks(&mut self, start_lba: Lba, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.block_size().assert_valid_block_buffer(dst);

        for (i, chunk) in dst.chunks_exact_mut(self.0.block_size()).enumerate() {
            self.0.read_block(start_lba.0 + i as u64, chunk)?;
        }

        Ok(())
    }

    fn write_blocks(&mut self, start_lba: Lba, src: &[u8]) -> Result<(), Self::Error> {
        self.block_size().assert_valid_block_buffer(src);

        for (i, chunk) in src.chunks_exact(self.0.block_size()).enumerate() {
            self.0.write_block(start_lba.0 + i as u64, chunk)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }
}

fn map_disk_error(err: DiskError<DevError>) -> DevError {
    match err {
        DiskError::BufferTooSmall => DevError::InvalidParam,
        DiskError::Overflow => DevError::BadState,
        DiskError::BlockSizeSmallerThanPartitionEntry => DevError::InvalidParam,
        DiskError::Io(e) => e,
    }
}

pub fn is_gpt_disk<T: BlockDriverOps>(inner: &mut T) -> DevResult<bool> {
    let mut disk = Disk::new(BlockDriverAdapter(inner)).map_err(map_disk_error)?;
    let mut block_buf = [0u8; 512];

    let primary_header = match disk.read_primary_gpt_header(&mut block_buf) {
        Ok(header) => header,
        Err(_) => return Ok(false),
    };

    Ok(primary_header.is_signature_valid())
}

pub fn find_partition_range<T, F>(inner: &mut T, mut predicate: F) -> DevResult<Option<Range<u64>>>
where
    T: BlockDriverOps,
    F: FnMut(usize, &GptPartitionEntry) -> bool,
{
    let mut disk = Disk::new(BlockDriverAdapter(inner)).map_err(map_disk_error)?;
    let mut block_buf = [0u8; 512];

    let primary_header = disk
        .read_primary_gpt_header(&mut block_buf)
        .map_err(map_disk_error)?;
    debug!("Primary GPT header: {primary_header}");

    let layout = primary_header.get_partition_entry_array_layout().unwrap();
    debug!("Partition entry array layout: {layout}");

    let secondary_header = disk
        .read_secondary_gpt_header(&mut block_buf)
        .map_err(map_disk_error)?;
    debug!("Secondary GPT header: {secondary_header}");

    for (i, part) in disk
        .gpt_partition_entry_array_iter(layout, &mut block_buf)
        .map_err(map_disk_error)?
        .enumerate()
    {
        let part = part.map_err(map_disk_error)?;
        if !part.is_used() {
            continue;
        }

        debug!("GPT partition[{i}]: {part}");
        if predicate(i, &part) {
            let range = part.starting_lba.to_u64()..part.ending_lba.to_u64() + 1;
            info!("Selected GPT partition: {part}");
            return Ok(Some(range));
        }
    }

    Ok(None)
}

/// A GPT partition wrapper around an existing block device.
pub struct GptPartitionDev<T> {
    inner: T,
    range: Range<u64>,
}

impl<T: BlockDriverOps> GptPartitionDev<T> {
    pub fn new(inner: T, range: Range<u64>) -> Self {
        Self { inner, range }
    }

    /// Creates a new GPT partition device from the given block storage device.
    ///
    /// The first partition that satisfies `predicate` will be selected.
    pub fn try_new<F>(mut inner: T, mut predicate: F) -> DevResult<Self>
    where
        F: FnMut(usize, &GptPartitionEntry) -> bool,
    {
        let range = find_partition_range(&mut inner, &mut predicate)?.ok_or(DevError::InvalidParam)?;
        Ok(Self::new(inner, range))
    }
}

impl<T: BlockDriverOps> BaseDriverOps for GptPartitionDev<T> {
    fn device_name(&self) -> &str {
        self.inner.device_name()
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }
}

impl<T: BlockDriverOps> BlockDriverOps for GptPartitionDev<T> {
    fn num_blocks(&self) -> u64 {
        self.range.end - self.range.start
    }

    fn block_size(&self) -> usize {
        self.inner.block_size()
    }

    fn read_block(&mut self, block_id: u64, buf: &mut [u8]) -> DevResult {
        if block_id >= self.num_blocks() {
            return Err(DevError::InvalidParam);
        }
        self.inner.read_block(self.range.start + block_id, buf)
    }

    fn write_block(&mut self, block_id: u64, buf: &[u8]) -> DevResult {
        if block_id >= self.num_blocks() {
            return Err(DevError::InvalidParam);
        }
        self.inner.write_block(self.range.start + block_id, buf)
    }

    fn flush(&mut self) -> DevResult {
        self.inner.flush()
    }
}