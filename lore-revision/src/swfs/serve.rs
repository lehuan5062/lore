// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
use std::cmp::min;
use std::ffi::CStr;
use std::ffi::c_void;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use parking_lot::Mutex;

use crate::errors::UnhandledError;
use crate::filter::Filter;
use crate::immutable;
use crate::interface::LoreString;
use crate::node::Node;
use crate::node::NodeBlock;
use crate::node::NodeLink;
use crate::node::ROOT_NODE;
use crate::node::SiblingCycleGuard;
use crate::repository::RepositoryContext;
use crate::state::State;
use lore_base::runtime::LORE_CONTEXT;
use crate::lore::execution_context;
use lore_base::runtime::runtime;
use crate::lore_error;
use crate::lore_info;
use crate::util::path::RelativePath;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(unused)]
#[allow(clippy::all)]
mod swfs {
    include!("swfs.rs");

    unsafe impl Send for SWFSInit {}
    unsafe impl Sync for SWFSInit {}

    unsafe impl Send for SWFSCallbacks {}
    unsafe impl Sync for SWFSCallbacks {}
}

#[error_set]
enum SWFSError {}

struct SWFSInstance {
    handle: swfs::SWFSHandle,
    repository: Arc<RepositoryContext>,
    state: Arc<State>,
    init: swfs::SWFSInit,
}

unsafe impl Send for SWFSInstance {}
unsafe impl Sync for SWFSInstance {}

struct SWFSDirectory {
    repository: Arc<RepositoryContext>,
    state: Arc<State>,
    #[allow(dead_code)]
    filter: Arc<Filter>,
    node_link: NodeLink,
}

static INSTANCE: Mutex<Option<Arc<SWFSInstance>>> = Mutex::new(None);

const WRITE_PATH: &CStr = c"D:\\swfs-write";
const MOUNT_POINT: &CStr = c"S:";

pub fn serve(_path: impl AsRef<Path>, repository: Arc<RepositoryContext>, state: Arc<State>) {
    let callbacks = swfs::SWFSCallbacks {
        open_file_read: Some(open_file_read),
        close_file_read: Some(close_file_read),
        read_file: Some(read_file),
        open_dir: Some(open_dir),
        fill_dir: Some(fill_dir),
        close_dir: Some(close_dir),
        is_file_ignored: Some(is_file_ignored),
        notify_open_write: Some(notify_open_write),
        notify_close_write: Some(notify_close_write),
        notify_create: Some(notify_create),
        notify_move: Some(notify_move),
        notify_delete: Some(notify_delete),
    };

    let instance = SWFSInstance {
        handle: std::ptr::null_mut(),
        repository: repository.clone(),
        state: state.clone(),
        init: swfs::SWFSInit {
            name: c"URC".to_bytes_with_nul().as_ptr().cast::<i8>(),
            write_dir: WRITE_PATH.to_bytes_with_nul().as_ptr().cast::<i8>(),
            mount_point: MOUNT_POINT.as_ptr().cast::<i8>(),
            flags: 0,
            perf_count_lg2: 0,
            num_threads: 0,
            log_hash_num_buckets_lg2: 0,
            callbacks,
        },
    };

    let instance = Arc::new(instance);

    {
        let mut lock = INSTANCE.lock();

        let instance_clone = instance.clone();
        thread::spawn(move || unsafe {
            println!("Calling swfsInitFn");
            if swfs::swfsInitFn(
                std::ptr::addr_of!(instance_clone.init) as *mut swfs::SWFSInit,
                std::ptr::addr_of!(instance_clone.handle) as *mut *mut c_void,
            ) == 0
            {
                panic!("Split-write FS init failed");
            }
            println!("Split-write FS thread exiting");
        });

        *lock = Some(instance);
    }

    println!("Split-write FS thread running");

    println!("Prefetching state fragments");
    let repository = repository.clone();
    let state = state.clone();
    let cache_task = lore_spawn!(async move {
        let _ = state.cache_fragments(repository).await;
        println!("Prefetching done");
    });
    drop(cache_task);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn verify_instance(swfs_handle: swfs::SWFSHandle) -> Result<Arc<SWFSInstance>, SWFSError> {
    let instance = {
        let lock = INSTANCE.lock();
        if let Some(locked_instance) = lock.as_ref() {
            locked_instance.clone()
        } else {
            return Err(SWFSError::internal("no SWFS instance found"));
        }
    };

    if instance.handle != swfs_handle {
        Err(SWFSError::internal("mismatching SWFS instance found"))
    } else {
        Ok(instance)
    }
}

extern "C" fn open_dir(swfs_handle: swfs::SWFSHandle, dir_handle: *mut swfs::SWFSFile) -> i32 {
    let cstr = unsafe { CStr::from_ptr((*dir_handle).filename) };
    let dir_name_string = unsafe { std::str::from_utf8_unchecked(cstr.to_bytes()) }
        .to_string()
        .replace('\\', "/");
    let dir_name = dir_name_string.trim_matches('/');

    let instance = match verify_instance(swfs_handle) {
        Ok(instance) => instance,
        Err(err) => {
            lore_error!("Failed open_dir: {err}");
            return 0;
        }
    };

    let repository = instance.repository.clone();
    let state = instance.state.clone();
    let filter = repository.filter.clone();
    let relative_path = RelativePath::new_from_initial_path(dir_name).unwrap_or_default();

    lore_info!("open_dir: {}", relative_path.as_str());

    runtime().block_on(LORE_CONTEXT.scope(execution_context(), async move {
        let Ok(node_link) = state
            .find_node_link(repository.clone(), relative_path.as_str())
            .await
        else {
            lore_error!(
                "Error finding path for open_dir: {}",
                relative_path.as_str()
            );
            return 0;
        };

        if !node_link.is_valid() && node_link.node != ROOT_NODE {
            lore_info!("Did not find path for open_dir: {}", relative_path.as_str());
            return 0;
        }

        // TODO(vri): UCS-19232 - Links: Handle link nodes in SWFS open_dir and open_file_read
        let directory = Box::new(SWFSDirectory {
            repository,
            state,
            filter,
            node_link,
        });

        unsafe {
            (*dir_handle).user_data = Box::into_raw(directory) as *mut c_void;
        }

        lore_info!(
            "Found node {} for open_dir {}",
            node_link.node,
            relative_path.as_str()
        );

        1
    }))
}

fn lore_time_to_ms_time(time_high: u32, time_low: u32) -> u64 {
    const MS_OFFSET_TIME: u64 = 116444736000000000;
    ((time_high as u64) << 32) + ((time_low as u64) * 10000) + MS_OFFSET_TIME
}

extern "C" fn fill_dir(
    swfs_handle: swfs::SWFSHandle,
    dir_handle: *mut swfs::SWFSFile,
    out_files: *mut *mut swfs::SWFSFile,
) -> u32 {
    let _instance = match verify_instance(swfs_handle) {
        Ok(instance) => instance,
        Err(err) => {
            lore_error!("Failed fill_dir: {err}");
            return 0;
        }
    };

    runtime().block_on(LORE_CONTEXT.scope(execution_context(), async move {
        let directory = unsafe {
            std::mem::ManuallyDrop::new(Box::from_raw(
                (*dir_handle).user_data as *mut SWFSDirectory,
            ))
        };
        let mut file_head: *mut swfs::SWFSFile = std::ptr::null_mut();
        let mut file_count: u32 = 0;

        let layout = unsafe {
            std::alloc::Layout::from_size_align_unchecked(
                size_of::<swfs::SWFSFile>(),
                align_of::<swfs::SWFSFile>(),
            )
        };

        lore_info!("fill_dir: {}", directory.node_link.node);

        if let Ok(node) = directory
            .state
            .node(directory.repository.clone(), directory.node_link.node)
            .await
        {
            let mut child_iter = if node.is_directory() {
                node.child()
            } else {
                None
            };
            let mut cycle = SiblingCycleGuard::new(directory.node_link.node);
            while let Some(child_id) = child_iter {
                let Ok((block, nametable)) = directory
                    .state
                    .block_and_nametable(directory.repository.clone(), NodeBlock::index(child_id))
                    .await
                else {
                    break;
                };

                let node = block.node(Node::index(child_id));

                if node
                    .walk_step(child_id, directory.node_link.node, &mut cycle)
                    .is_err()
                {
                    break;
                }

                let node_name = directory.state.node_name_direct(&node, &nametable);

                let name_string = Box::into_raw(Box::new(LoreString::from(node_name)));

                unsafe {
                    let file = std::alloc::alloc(layout) as *mut swfs::SWFSFile;
                    let access_time = lore_time_to_ms_time(node.mtime_high, node.child_mtime_node);
                    *file = swfs::SWFSFile {
                        filename: (*name_string).string as *mut i8,
                        file_attributes: if node.is_file() {
                            128 /* FILE_ATTRIBUTE_NORMAL */
                        } else {
                            16 /* FILE_ATTRIBUTE_DIRECTORY */
                        },
                        access: 0x1F01FF, /* FILE_ALL_ACCESS */
                        size: node.size,
                        creation_time: access_time,
                        last_access_time: access_time,
                        change_time: access_time,
                        user_data: name_string.cast::<c_void>(),
                        next: file_head,
                    };

                    if node.is_file() {
                        lore_info!("Add file: {}", node_name);
                    } else {
                        lore_info!("Add dir: {}", node_name);
                    }

                    file_head = file;
                    file_count += 1;
                }

                child_iter = node.sibling();
            }
        }

        lore_info!("fill_dir done: {file_count}");

        unsafe {
            *out_files = file_head;
        }

        file_count
    }))
}

extern "C" fn close_dir(
    _swfs_handle: swfs::SWFSHandle,
    dir_handle: *mut swfs::SWFSFile,
    files: *mut swfs::SWFSFile,
    num_files: u32,
) {
    let directory = unsafe { Box::from_raw((*dir_handle).user_data as *mut SWFSDirectory) };

    lore_info!("close_dir: {}", directory.node_link.node);

    if num_files == 0 {
        return;
    }

    let mut file = files;

    let layout = unsafe {
        std::alloc::Layout::from_size_align_unchecked(
            size_of::<swfs::SWFSFile>(),
            align_of::<swfs::SWFSFile>(),
        )
    };

    while !file.is_null() {
        unsafe {
            let next = (*file).next;

            drop(Box::from_raw((*file).user_data.cast::<LoreString>()));
            std::alloc::dealloc(file as *mut u8, layout);

            file = next;
        }
    }

    drop(directory);
}

extern "C" fn open_file_read(swfs_handle: swfs::SWFSHandle, swfs_file: *mut swfs::SWFSFile) -> i32 {
    let instance = match verify_instance(swfs_handle) {
        Ok(instance) => instance,
        Err(err) => {
            lore_error!("Failed open_file_read: {err}");
            return 0;
        }
    };

    let cstr = unsafe { CStr::from_ptr((*swfs_file).filename) };
    let file_name_string = unsafe { std::str::from_utf8_unchecked(cstr.to_bytes()) }
        .to_string()
        .replace('\\', "/");
    let file_name = file_name_string.trim_matches('/');

    lore_info!("open_file_read: {file_name}");

    let repository = instance.repository.clone();
    let state = instance.state.clone();

    let relative_path = RelativePath::new_from_initial_path(file_name).unwrap_or_default();

    runtime().block_on(LORE_CONTEXT.scope(execution_context(), async move {
        let Ok(node_link) = state
            .find_node_link(repository.clone(), relative_path.as_str())
            .await
        else {
            lore_error!(
                "Error finding path for open_file_read: {}",
                relative_path.as_str()
            );
            return 0;
        };

        // TODO(vri): UCS-19232 - Links: Handle link nodes in SWFS open_dir and open_file_read
        if let Ok(node) = state.node(repository.clone(), node_link.node).await {
            if node.is_directory() {
                lore_info!(
                    "Node is directory in open_file_read, return 0: {}",
                    relative_path.as_str()
                );
                return 0;
            }

            unsafe {
                let access_time = lore_time_to_ms_time(node.mtime_high, node.child_mtime_node);

                (*swfs_file).file_attributes = 128 /* FILE_ATTRIBUTE_NORMAL */;
                (*swfs_file).size = node.size;
                (*swfs_file).creation_time = access_time;
                (*swfs_file).last_access_time = access_time;
                (*swfs_file).change_time = access_time;

                (*swfs_file).user_data = Box::into_raw(Box::new(node)).cast();
            }

            lore_info!(
                "Found node size {} for open_file_read: {}",
                node.size,
                relative_path.as_str()
            );

            1
        } else {
            lore_info!(
                "Error loading node for open_file_read: {}",
                relative_path.as_str()
            );

            0
        }
    }))
}

extern "C" fn close_file_read(_swfs_handle: swfs::SWFSHandle, swfs_file: *mut swfs::SWFSFile) {
    let cstr = unsafe { CStr::from_ptr((*swfs_file).filename) };
    let file_name_string = unsafe { std::str::from_utf8_unchecked(cstr.to_bytes()) }
        .to_string()
        .replace('\\', "/");
    let file_name = file_name_string.trim_matches('/');

    lore_info!("close_file_read: {file_name}");

    let node = unsafe { Box::from_raw((*swfs_file).user_data.cast::<Node>()) };
    drop(node);
}

extern "C" fn read_file(
    swfs_handle: swfs::SWFSHandle,
    swfs_file: *mut swfs::SWFSFile,
    out_buffer: *mut c_void,
    read_offset: u64,
    num_bytes_to_read: u64,
) -> u64 {
    let instance = match verify_instance(swfs_handle) {
        Ok(instance) => instance,
        Err(err) => {
            lore_error!("Failed read_file: {err}");
            return 0;
        }
    };

    let cstr = unsafe { CStr::from_ptr((*swfs_file).filename) };
    let file_name_string = unsafe { std::str::from_utf8_unchecked(cstr.to_bytes()) }
        .to_string()
        .replace('\\', "/");
    let file_name = file_name_string.trim_matches('/');

    lore_info!("read_file: {file_name}");

    let repository = instance.repository.clone();
    let node = unsafe {
        std::mem::ManuallyDrop::new(Box::from_raw((*swfs_file).user_data.cast::<Node>()))
    };

    let offset = min(read_offset, node.size);
    let byte_count = min(num_bytes_to_read, node.size - offset);

    lore_info!(
        "read_file {num_bytes_to_read} @ offset {read_offset} - capped to {byte_count} @ offset {offset}"
    );

    if byte_count == 0 {
        return 0;
    }

    runtime().block_on(LORE_CONTEXT.scope(execution_context(), async move {
        let target =
            unsafe { std::slice::from_raw_parts_mut(out_buffer.cast::<u8>(), byte_count as usize) };

        let range = (offset as usize)..((offset + byte_count) as usize);
        let options = immutable::read_options_from_repository(&repository).with_cache();
        let result = immutable::read_into(
            repository.clone(),
            node.address,
            Some(range),
            target,
            options,
        )
        .await;

        if let Err(err) = result {
            lore_error!("Failed to read {byte_count} @ offset {offset}: {err}");
            0
        } else {
            lore_info!("Read {byte_count} bytes @ offset {offset} from {file_name}");
            byte_count as u64
        }
    }))
}

unsafe extern "C" fn is_file_ignored(
    _swfs_handle: swfs::SWFSHandle,
    _path: *const swfs::swfs_utf8,
) -> i32 {
    0
}

unsafe extern "C" fn notify_open_write(
    _swfs_handle: swfs::SWFSHandle,
    _swfs_file: *mut swfs::SWFSFile,
) {
}

unsafe extern "C" fn notify_close_write(
    _swfs_handle: swfs::SWFSHandle,
    _swfs_file: *mut swfs::SWFSFile,
) {
}

unsafe extern "C" fn notify_create(
    _swfs_handle: swfs::SWFSHandle,
    _swfs_file: *mut swfs::SWFSFile,
) {
}

unsafe extern "C" fn notify_move(
    _swfs_handle: swfs::SWFSHandle,
    _old_file: *mut swfs::SWFSFile,
    _new_file: *mut swfs::SWFSFile,
) -> i32 {
    0
}

unsafe extern "C" fn notify_delete(
    _swfs_handle: swfs::SWFSHandle,
    _swfs_file: *mut swfs::SWFSFile,
    _arg1: swfs::SWFSDeleteStatus_Enum,
) -> swfs::SWFSDeleteResult_Enum {
    swfs::SWFSDeleteResult_Enum_SWFSDeleteResult_Continue
}
