extern crate alloc;
extern crate log;

use alloc::string;
use alloc::vec;
use bitflags::Flags;
use core::marker::PhantomData;
use core::mem::size_of;
use core::str;
use core::*;

use super::ext4_defs::*;
use crate::consts::*;
use crate::prelude::*;
use crate::utils::*;

pub(crate) const BASE_OFFSET: usize = 1024;
pub(crate) const BLOCK_SIZE: usize = 4096;

// 定义ext4_ext_binsearch函数，接受一个指向ext4_extent_path的可变引用和一个逻辑块号，返回一个布尔值，表示是否找到了对应的extent
pub fn ext4_ext_binsearch(path: &mut Ext4ExtentPath, block: u32) -> bool {
    // 获取extent header的引用
    let eh = unsafe { &*path.header };

    if eh.entries_count == 0 {
        /*
         * this leaf is empty:
         * we get such a leaf in split/add case
         */
        false;
    }

    // 定义左右两个指针，分别指向第一个和最后一个extent
    let mut l = unsafe { ext4_first_extent(eh).add(1) };
    let mut r = unsafe { ext4_last_extent(eh) };

    // 如果extent header中没有有效的entry，直接返回false
    if eh.entries_count == 0 {
        return false;
    }
    // 使用while循环进行二分查找
    while l <= r {
        // 计算中间指针
        let m = unsafe { l.add((r as usize - l as usize) / 2) };
        // 获取中间指针所指向的extent的引用
        let ext = unsafe { &*m };
        // 比较逻辑块号和extent的第一个块号
        if block < ext.first_block {
            // 如果逻辑块号小于extent的第一个块号，说明目标在左半边，将右指针移动到中间指针的左边
            r = unsafe { m.sub(1) };
        } else {
            // 如果逻辑块号大于或等于extent的第一个块号，说明目标在右半边，将左指针移动到中间指针的右边
            l = unsafe { m.add(1) };
        }
    }
    // 循环结束后，将path的extent字段设置为左指针的前一个位置
    path.extent = unsafe { l.sub(1) };
    // 返回true，表示找到了对应的extent
    true
}

pub trait BlockDevice: Send + Sync + Any + Debug {
    fn read_offset(&self, offset: usize) -> Vec<u8>;
    fn write_offset(&self, offset: usize, data: &[u8]);
}

impl dyn BlockDevice {
    pub fn downcast_ref<T: BlockDevice>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

#[derive(Debug)]
pub struct Ext4 {
    pub block_device: Arc<dyn BlockDevice>,
    pub super_block: Ext4Superblock,
    pub block_groups: Vec<Ext4BlockGroup>,
    pub inodes_per_group: u32,
    pub blocks_per_group: u32,
    pub inode_size: usize,
    pub self_ref: Weak<Self>,
    pub mount_point: Ext4MountPoint,
}

impl Ext4 {
    /// Opens and loads an Ext4 from the `block_device`.
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Arc<Self> {
        // Load the superblock
        // TODO: if the main superblock is corrupted, should we load the backup?
        let raw_data = block_device.read_offset(BASE_OFFSET);
        let super_block = Ext4Superblock::try_from(raw_data).unwrap();

        println!("super_block: {:x?}", super_block);
        let inodes_per_group = super_block.inodes_per_group();
        let blocks_per_group = super_block.blocks_per_group();
        let inode_size = super_block.inode_size();

        // Load the block groups information
        let load_block_groups = |fs: Weak<Ext4>,
                                 block_device: Arc<dyn BlockDevice>|
         -> Result<Vec<Ext4BlockGroup>> {
            let block_groups_count = super_block.block_groups_count() as usize;
            let mut block_groups = Vec::with_capacity(block_groups_count);
            for idx in 0..block_groups_count {
                let block_group = Ext4BlockGroup::load(block_device.clone(), &super_block, idx).unwrap();
                block_groups.push(block_group);
            }
            Ok(block_groups)
        };

        let mount_point = Ext4MountPoint::new("/");

        let ext4 = Arc::new_cyclic(|weak_ref| Self {
            super_block: super_block,
            inodes_per_group: inodes_per_group,
            blocks_per_group: blocks_per_group,
            inode_size: inode_size as usize,
            block_groups: load_block_groups(weak_ref.clone(), block_device.clone()).unwrap(),
            block_device,
            self_ref: weak_ref.clone(),
            mount_point: mount_point,
        });

        ext4
    }

    // 使用libc库定义的常量
    fn ext4_parse_flags(&self, flags: &str) -> Result<u32> {
        let flag = flags.parse::<Ext4OpenFlags>().unwrap(); // 从字符串转换为标志
        let file_flags = match flag {
            Ext4OpenFlags::ReadOnly => O_RDONLY,
            Ext4OpenFlags::WriteOnly => O_WRONLY,
            Ext4OpenFlags::WriteCreateTrunc => O_WRONLY | O_CREAT | O_TRUNC,
            Ext4OpenFlags::WriteCreateAppend => O_WRONLY | O_CREAT | O_APPEND,
            Ext4OpenFlags::ReadWrite => O_RDWR,
            Ext4OpenFlags::ReadWriteCreateTrunc => O_RDWR | O_CREAT | O_TRUNC,
            Ext4OpenFlags::ReadWriteCreateAppend => O_RDWR | O_CREAT | O_APPEND,
        };
        Ok(file_flags as u32) // 转换为数值
    }

    // start transaction
    pub fn ext4_trans_start(&self) {}

    // stop transaction
    pub fn ext4_trans_abort(&self) {}

    pub fn ext4_open(&self, file: &mut Ext4File, path: &str, flags: &str, file_expect: bool) {
        let mut iflags = 0;
        let mut filetype = DirEntryType::EXT4_DE_UNKNOWN;

        // get mount point
        let mut ptr = Box::new(self.mount_point.clone());
        file.mp = Box::as_mut(&mut ptr) as *mut Ext4MountPoint;

        // get open flags
        iflags = self.ext4_parse_flags(flags).unwrap();

        // file for dir
        if file_expect {
            filetype = DirEntryType::EXT4_DE_REG_FILE;
        } else {
            filetype = DirEntryType::EXT4_DE_DIR;
        }

        if iflags & O_CREAT != 0 {
            self.ext4_trans_start();
        }
        self.ext4_generic_open(file, path, iflags, filetype.bits(), None);
    }

    pub fn ext4_generic_open(
        &self,
        file: &mut Ext4File,
        path: &str,
        iflags: u32,
        ftype: u8,
        parent_inode: Option<&mut Ext4InodeRef>,
    ) {
        let mut is_goal = false;

        let mp: &Ext4MountPoint = &self.mount_point;

        let mp_name = mp.mount_name.as_bytes();

        let mut data: Vec<u8> = Vec::with_capacity(BLOCK_SIZE);
        let ext4_blk = Ext4Block {
            disk_block_id: 0,
            block_data: &mut data,
            dirty: true,
        };
        let mut de = Ext4DirEntry::default();
        let mut dir_search_result = Ext4DirSearchResult::new(ext4_blk, de);
        let path_skip_mount = ext4_path_skip(path, core::str::from_utf8(mp_name).unwrap());

        file.flags = iflags;

        // load root inode
        let root_inode_ref = Ext4InodeRef::get_inode_ref(self.self_ref.clone(), 2);

        if !parent_inode.is_none() {
            parent_inode.unwrap().inode_num = root_inode_ref.inode_num;
        }

        let mut len = ext4_path_check(path_skip_mount, &mut is_goal);

        let mut serach_path = path_skip_mount;

        loop {
            len = ext4_path_check(&serach_path, &mut is_goal);

            let r = ext4_dir_find_entry(
                &root_inode_ref,
                serach_path,
                len as u32,
                &mut dir_search_result,
            );

            if r != EOK {
                ext4_dir_destroy_result();

                let mut child_inode_ref = Ext4InodeRef::new(self.self_ref.clone());

                let r = ext4_fs_alloc_inode(&mut child_inode_ref);

                if r != EOK {
                    break;
                }

                ext4_fs_inode_blocks_init(&mut child_inode_ref);

                let r = ext4_link();

                if r != EOK {
                    /*Fail. Free new inode.*/
                    break;
                }

                ext4_fs_put_inode_ref(&mut child_inode_ref);
            }
        }
    }
}

pub fn ext4_fs_put_inode_ref(inode_ref: &mut Ext4InodeRef) {
    inode_ref.inner.write_back_inode();
}

pub fn ext4_link() -> usize {
    0
}

pub fn ext4_fs_inode_blocks_init(inode_ref: &mut Ext4InodeRef) {}

pub fn ext4_fs_alloc_inode(child_inode_ref: &mut Ext4InodeRef) -> usize {
    0
}
pub fn ext4_dir_destroy_result() {}

pub fn ext4_dir_find_entry(
    parent: &Ext4InodeRef,
    name: &str,
    name_len: u32,
    result: &mut Ext4DirSearchResult,
) -> usize {
    0
}
