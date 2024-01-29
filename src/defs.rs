use bitflags::bitflags;
use core::mem::size_of;
use std::fmt::Error;

use super::*;
use crate::prelude::*;

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

/// 文件描述符
pub struct Ext4FileNew {
    /// 挂载点句柄
    pub mp: *mut Ext4MountPoint,
    /// 文件 inode id
    pub inode: u32,
    /// 打开标志
    pub flags: u32,
    /// 文件大小
    pub fsize: u64,
    /// 实际文件位置
    pub fpos: u64,
}

impl Ext4FileNew {
    pub fn new() -> Self {
        Self {
            mp: core::ptr::null_mut(),
            inode: 0,
            flags: 0,
            fsize: 0,
            fpos: 0,
        }
    }
}

// 结构体表示超级块
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ext4Superblock {
    inodes_count: u32,             // 节点数
    blocks_count_lo: u32,          // 块数
    reserved_blocks_count_lo: u32, // 保留块数
    free_blocks_count_lo: u32,     // 空闲块数
    free_inodes_count: u32,        // 空闲节点数
    first_data_block: u32,         // 第一个数据块
    log_block_size: u32,           // 块大小
    log_cluster_size: u32,         // 废弃的片段大小
    blocks_per_group: u32,         // 每组块数
    frags_per_group: u32,          // 废弃的每组片段数
    inodes_per_group: u32,         // 每组节点数
    mount_time: u32,               // 挂载时间
    write_time: u32,               // 写入时间
    mount_count: u16,              // 挂载次数
    max_mount_count: u16,          // 最大挂载次数
    magic: u16,                    // 魔数，0xEF53
    state: u16,                    // 文件系统状态
    errors: u16,                   // 检测到错误时的行为
    minor_rev_level: u16,          // 次版本号
    last_check_time: u32,          // 最后检查时间
    check_interval: u32,           // 检查间隔
    creator_os: u32,               // 创建者操作系统
    rev_level: u32,                // 版本号
    def_resuid: u16,               // 保留块的默认uid
    def_resgid: u16,               // 保留块的默认gid

    // 仅适用于EXT4_DYNAMIC_REV超级块的字段
    first_inode: u32,            // 第一个非保留节点
    inode_size: u16,             // 节点结构的大小
    block_group_index: u16,      // 此超级块的块组索引
    features_compatible: u32,    // 兼容特性集
    features_incompatible: u32,  // 不兼容特性集
    features_read_only: u32,     // 只读兼容特性集
    uuid: [u8; 16],              // 卷的128位uuid
    volume_name: [u8; 16],       // 卷名
    last_mounted: [u8; 64],      // 最后挂载的目录
    algorithm_usage_bitmap: u32, // 用于压缩的算法

    // 性能提示。只有当EXT4_FEATURE_COMPAT_DIR_PREALLOC标志打开时，才进行目录预分配
    s_prealloc_blocks: u8,      // 尝试预分配的块数
    s_prealloc_dir_blocks: u8,  // 为目录预分配的块数
    s_reserved_gdt_blocks: u16, // 在线增长时每组保留的描述符数

    // 如果EXT4_FEATURE_COMPAT_HAS_JOURNAL设置，表示支持日志
    journal_uuid: [u8; 16],    // 日志超级块的UUID
    journal_inode_number: u32, // 日志文件的节点号
    journal_dev: u32,          // 日志文件的设备号
    last_orphan: u32,          // 待删除节点的链表头
    hash_seed: [u32; 4],       // HTREE散列种子
    default_hash_version: u8,  // 默认的散列版本
    journal_backup_type: u8,
    desc_size: u16,            // 组描述符的大小
    default_mount_opts: u32,   // 默认的挂载选项
    first_meta_bg: u32,        // 第一个元数据块组
    mkfs_time: u32,            // 文件系统创建的时间
    journal_blocks: [u32; 17], // 日志节点的备份

    // 如果EXT4_FEATURE_COMPAT_64BIT设置，表示支持64位
    blocks_count_hi: u32,          // 块数
    reserved_blocks_count_hi: u32, // 保留块数
    free_blocks_count_hi: u32,     // 空闲块数
    min_extra_isize: u16,          // 所有节点至少有#字节
    want_extra_isize: u16,         // 新节点应该保留#字节
    flags: u32,                    // 杂项标志
    raid_stride: u16,              // RAID步长
    mmp_interval: u16,             // MMP检查的等待秒数
    mmp_block: u64,                // 多重挂载保护的块
    raid_stripe_width: u32,        // 所有数据磁盘上的块数（N * 步长）
    log_groups_per_flex: u8,       // FLEX_BG组的大小
    checksum_type: u8,
    reserved_pad: u16,
    kbytes_written: u64,          // 写入的千字节数
    snapshot_inum: u32,           // 活动快照的节点号
    snapshot_id: u32,             // 活动快照的顺序ID
    snapshot_r_blocks_count: u64, // 为活动快照的未来使用保留的块数
    snapshot_list: u32,           // 磁盘上快照列表的头节点号
    error_count: u32,             // 文件系统错误的数目
    first_error_time: u32,        // 第一次发生错误的时间
    first_error_ino: u32,         // 第一次发生错误的节点号
    first_error_block: u64,       // 第一次发生错误的块号
    first_error_func: [u8; 32],   // 第一次发生错误的函数
    first_error_line: u32,        // 第一次发生错误的行号
    last_error_time: u32,         // 最近一次发生错误的时间
    last_error_ino: u32,          // 最近一次发生错误的节点号
    last_error_line: u32,         // 最近一次发生错误的行号
    last_error_block: u64,        // 最近一次发生错误的块号
    last_error_func: [u8; 32],    // 最近一次发生错误的函数
    mount_opts: [u8; 64],
    usr_quota_inum: u32,       // 用于跟踪用户配额的节点
    grp_quota_inum: u32,       // 用于跟踪组配额的节点
    overhead_clusters: u32,    // 文件系统中的开销块/簇
    backup_bgs: [u32; 2],      // 有sparse_super2超级块的组
    encrypt_algos: [u8; 4],    // 使用的加密算法
    encrypt_pw_salt: [u8; 16], // 用于string2key算法的盐
    lpf_ino: u32,              // lost+found节点的位置
    padding: [u32; 100],       // 块的末尾的填充
    checksum: u32,             // crc32c(superblock)
}

impl TryFrom<Vec<u8>> for Ext4Superblock {
    type Error = u64;
    fn try_from(value: Vec<u8>) -> Result<Self> {
        let data = &value[..size_of::<Ext4Superblock>()];
        unsafe { core::ptr::read(data.as_ptr() as *const _) }
    }
}

impl Ext4Superblock {
    pub fn sync_super_block_to_disk(&self, block_device: &dyn BlockDevice) -> Result<()> {
        let data = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Ext4Superblock>())
        };
        block_device.write_offset(BASE_OFFSET, data);
        Ok(())
    }
}

impl Ext4Superblock {
    /// Returns the size of inode structure.
    pub fn inode_size(&self) -> u16 {
        self.inode_size
    }
    /// Returns total number of inodes.
    pub fn total_inodes(&self) -> u32 {
        self.inodes_count
    }

    /// Returns the number of blocks in each block group.
    pub fn blocks_per_group(&self) -> u32 {
        self.blocks_per_group
    }

    /// Returns the number of inodes in each block group.
    pub fn inodes_per_group(&self) -> u32 {
        self.inodes_per_group
    }

    /// Returns the number of block groups.
    pub fn block_groups_count(&self) -> u32 {
        (self.blocks_count_hi.to_le() as u32) << 32 | self.blocks_count_lo
    }

    pub fn desc_size(&self) -> u16 {
        let size = self.desc_size;

        if size < EXT4_MIN_BLOCK_GROUP_DESCRIPTOR_SIZE {
            return EXT4_MIN_BLOCK_GROUP_DESCRIPTOR_SIZE as u16;
        } else {
            size
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Ext4Inode {
    pub mode: u16,
    pub uid: u16,
    pub size: u32,
    pub atime: u32,
    pub ctime: u32,
    pub mtime: u32,
    pub dtime: u32,
    pub gid: u16,
    pub links_count: u16,
    pub blocks: u32,
    pub flags: u32,
    pub osd1: u32,
    pub block: [u32; 15],
    pub generation: u32,
    pub file_acl: u32,
    pub size_hi: u32,
    pub faddr: u32,   /* Obsoleted fragment address */
    pub osd2: Linux2, // 操作系统相关的字段2

    pub i_extra_isize: u16,
    pub i_checksum_hi: u16,  // crc32c(uuid+inum+inode) BE
    pub i_ctime_extra: u32,  // 额外的修改时间（nsec << 2 | epoch）
    pub i_mtime_extra: u32,  // 额外的文件修改时间（nsec << 2 | epoch）
    pub i_atime_extra: u32,  // 额外的访问时间（nsec << 2 | epoch）
    pub i_crtime: u32,       // 文件创建时间
    pub i_crtime_extra: u32, // 额外的文件创建时间（nsec << 2 | epoch）
    pub i_version_hi: u32,   // 64位版本的高32位
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Linux2 {
    pub l_i_blocks_high: u16, // 原来是l_i_reserved1
    pub l_i_file_acl_high: u16,
    pub l_i_uid_high: u16,    // 这两个字段
    pub l_i_gid_high: u16,    // 原来是reserved2[0]
    pub l_i_checksum_lo: u16, // crc32c(uuid+inum+inode) LE
    pub l_i_reserved: u16,
}

impl TryFrom<&[u8]> for Ext4Inode {
    type Error = u64;
    fn try_from(data: &[u8]) -> Result<Self> {
        let data = &data[..size_of::<Ext4Inode>()];
        unsafe { core::ptr::read(data.as_ptr() as *const _) }
    }
}

impl Ext4Inode {
    fn get_checksum(&self, super_block: &Ext4Superblock) -> u32 {
        let inode_size = super_block.inode_size;
        let mut v: u32 = self.osd2.l_i_checksum_lo as u32;
        if inode_size > 128 {
            v |= (self.i_checksum_hi as u32) << 16;
        }
        v
    }
}

impl Ext4Inode {
    pub fn get_inode_disk_pos(
        &self,
        super_block: &Ext4Superblock,
        block_device: &dyn BlockDevice,
        inode_id: u32,
    ) -> usize {
        let inodes_per_group = super_block.inodes_per_group;
        let inode_size = super_block.inode_size;
        let group = (inode_id - 1) / inodes_per_group;
        let index = (inode_id - 1) % inodes_per_group;

        let mut inode_table_blk_num =
            Ext4BlockGroup::load(block_device, super_block, group as usize).unwrap();
        let mut offset = inode_table_blk_num * BLOCK_SIZE + index * inode_size as u32;
        offset
    }

    pub fn sync_inode_to_disk(
        &self,
        block_device: &dyn BlockDevice,
        super_block: &Ext4Superblock,
        inode_id: u32,
    ) -> Result<()> {
        let disk_pos = self.get_inode_disk_pos(super_block, block_device, inode_id);
        let data = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Ext4Inode>())
        };
        block_device.write_offset(disk_pos, data);

        Ok(())
    }

    pub fn get_inode_checksum(&mut self, inode_id: u32, super_block: &Ext4Superblock) -> u32 {
        let inode_size = super_block.inode_size();

        let orig_checksum = self.get_checksum(super_block);
        let mut checksum = 0;

        let ino_index = inode_id as u32;
        let ino_gen = self.generation;

        // Preparation: temporarily set bg checksum to 0
        self.osd2.l_i_checksum_lo = 0;
        self.i_checksum_hi = 0;

        checksum = ext4_crc32c(
            EXT4_CRC32_INIT,
            &super_block.uuid,
            super_block.uuid.len() as u32,
        );
        checksum = ext4_crc32c(checksum, &ino_index.to_le_bytes(), 4);
        checksum = ext4_crc32c(checksum, &ino_gen.to_le_bytes(), 4);

        // cast self to &[u8]
        // attention checksum size here is 0x100 inode_size is 0x97
        let self_bytes =
            unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 0x100 as usize) };

        // inode checksum
        checksum = ext4_crc32c(checksum, self_bytes, inode_size as u32);

        self.set_inode_checksum_value(super_block, inode_id, checksum);

        if inode_size == 128 {
            checksum &= 0xFFFF;
        }

        checksum
    }

    pub fn set_inode_checksum_value(
        &mut self,
        super_block: &Ext4Superblock,
        inode_id: u32,
        checksum: u32,
    ) {
        let inode_size = super_block.inode_size();
        // let csum = self.get_inode_checksum(inode_id, super_block);

        self.osd2.l_i_checksum_lo = ((checksum << 16) >> 16) as u16;
        if inode_size > 128 {
            self.i_checksum_hi = (checksum >> 16) as u16;
        }
    }

    pub fn set_inode_checksum(&mut self, super_block: &Ext4Superblock, inode_id: u32) {
        let inode_size = super_block.inode_size();
        let checksum = self.get_inode_checksum(inode_id, super_block);

        self.osd2.l_i_checksum_lo = ((checksum << 16) >> 16) as u16;
        if inode_size > 128 {
            self.i_checksum_hi = (checksum >> 16) as u16;
        }
    }

    pub fn sync_inode_to_disk_with_csum(
        &mut self,
        block_device: &dyn BlockDevice,
        super_block: &Ext4Superblock,
        inode_id: u32,
    ) -> Result<()> {
        self.set_inode_checksum(super_block, inode_id);
        self.sync_inode_to_disk(block_device, super_block, inode_id)
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct Ext4BlockGroup {
    block_bitmap_lo: u32,            // 块位图块
    inode_bitmap_lo: u32,            // 节点位图块
    inode_table_first_block_lo: u32, // 节点表块
    free_blocks_count_lo: u16,       // 空闲块数
    free_inodes_count_lo: u16,       // 空闲节点数
    used_dirs_count_lo: u16,         // 目录数
    flags: u16,                      // EXT4_BG_flags (INODE_UNINIT, etc)
    exclude_bitmap_lo: u32,          // 快照排除位图
    block_bitmap_csum_lo: u16,       // crc32c(s_uuid+grp_num+bbitmap) LE
    inode_bitmap_csum_lo: u16,       // crc32c(s_uuid+grp_num+ibitmap) LE
    itable_unused_lo: u16,           // 未使用的节点数
    checksum: u16,                   // crc16(sb_uuid+group+desc)

    block_bitmap_hi: u32,            // 块位图块 MSB
    inode_bitmap_hi: u32,            // 节点位图块 MSB
    inode_table_first_block_hi: u32, // 节点表块 MSB
    free_blocks_count_hi: u16,       // 空闲块数 MSB
    free_inodes_count_hi: u16,       // 空闲节点数 MSB
    used_dirs_count_hi: u16,         // 目录数 MSB
    itable_unused_hi: u16,           // 未使用的节点数 MSB
    exclude_bitmap_hi: u32,          // 快照排除位图 MSB
    block_bitmap_csum_hi: u16,       // crc32c(s_uuid+grp_num+bbitmap) BE
    inode_bitmap_csum_hi: u16,       // crc32c(s_uuid+grp_num+ibitmap) BE
    reserved: u32,                   // 填充
}

impl TryFrom<&[u8]> for Ext4BlockGroup {
    type Error = u64;
    fn try_from(data: &[u8]) -> Result<Self> {
        let data = &data[..size_of::<Ext4BlockGroup>()];
        unsafe { core::ptr::read(data.as_ptr() as *const _) }
    }
}

impl Ext4BlockGroup {
    pub fn sync_block_group_to_disk(
        &self,
        block_device: &dyn BlockDevice,
        bgid: usize,
        super_block: &Ext4Superblock,
    ) -> Result<()> {
        let dsc_cnt = BLOCK_SIZE / super_block.desc_size as usize;
        let dsc_per_block = dsc_cnt;
        let dsc_id = bgid / dsc_cnt;
        let first_meta_bg = super_block.first_meta_bg;
        let first_data_block = super_block.first_data_block;
        let block_id = first_data_block as usize + dsc_id + 1;
        let offset = (bgid % dsc_cnt) * super_block.desc_size as usize;

        let data = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Ext4BlockGroup>())
        };
        block_device.write_offset(block_id * BLOCK_SIZE + offset, data);
        Ok(())
    }

    pub fn get_block_group_checksum(&mut self, bgid: u32, super_block: &Ext4Superblock) -> u16 {
        let desc_size = super_block.desc_size();

        let mut orig_checksum = 0;
        let mut checksum = 0;

        orig_checksum = self.checksum;

        // 准备：暂时将bg校验和设为0
        self.checksum = 0;

        // uuid checksum
        checksum = ext4_crc32c(
            EXT4_CRC32_INIT,
            &super_block.uuid,
            super_block.uuid.len() as u32,
        );

        // bgid checksum
        checksum = ext4_crc32c(checksum, &bgid.to_le_bytes(), 4);

        // cast self to &[u8]
        let self_bytes =
            unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, 0x40 as usize) };

        // bg checksum
        checksum = ext4_crc32c(checksum, self_bytes, desc_size as u32);

        self.checksum = orig_checksum;

        let crc = (checksum & 0xFFFF) as u16;

        crc
    }

    pub fn set_block_group_checksum(&mut self, bgid: u32, super_block: &Ext4Superblock) {
        let csum = self.get_block_group_checksum(bgid, super_block);
        self.checksum = csum;
    }

    pub fn sync_to_disk_with_csum(
        &mut self,
        block_device: &dyn BlockDevice,
        bgid: usize,
        super_block: &Ext4Superblock,
    ) -> Result<()> {
        self.set_block_group_checksum(bgid as u32, super_block);
        self.sync_block_group_to_disk(block_device, bgid, super_block)
    }
}

impl Ext4BlockGroup {
    pub fn load(
        block_device: &dyn BlockDevice,
        super_block: &Ext4Superblock,
        block_group_idx: usize,
        // fs: Weak<Ext4>,
    ) -> Result<Self> {
        let dsc_cnt = BLOCK_SIZE / super_block.desc_size as usize;
        let dsc_per_block = dsc_cnt;
        let dsc_id = block_group_idx / dsc_cnt;
        let first_meta_bg = super_block.first_meta_bg;
        let first_data_block = super_block.first_data_block;

        let block_id = first_data_block as usize + dsc_id + 1;
        let offset = (block_group_idx % dsc_cnt) * super_block.desc_size as usize;

        let data = block_device.read_offset(block_id * BLOCK_SIZE);

        let block_group_data =
            &data[offset as usize..offset as usize + size_of::<Ext4BlockGroup>()];

        let bg = Ext4BlockGroup::try_from(block_group_data);
        bg
    }
}
pub struct Inode {
    ino: u32,
    block_group_idx: usize,
    inner: Inner,
    fs: Weak<Ext4>,
}

impl Inode {
    pub fn fs(&self) -> Arc<Ext4> {
        self.fs.upgrade().unwrap()
    }
}

struct Inner {
    inode: Ext4Inode,
    weak_self: Weak<Inode>,
}

impl Inner {
    pub fn inode(&self) -> Arc<Inode> {
        self.weak_self.upgrade().unwrap()
    }
}

/**@brief   Mount point descriptor.*/
pub struct Ext4MountPoint {
    /**@brief   Mount done flag.*/
    pub mounted: bool,
    /**@brief   Mount point name (@ref ext4_mount)*/
    pub mount_name: [char; 33],

    pub mount_name_string: String,
}
