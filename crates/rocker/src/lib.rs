#[macro_use]
extern crate lazy_static;
extern crate rocksdb;
#[macro_use]
extern crate rustler;

use rocksdb::{DB, DBCompactionStyle, Options};
use rustler::{NifEncoder, NifEnv, NifResult, NifTerm};
use rustler::resource::ResourceArc;
use rustler::schedule::NifScheduleFlags;
use rustler::types::map::NifMapIterator;
use std::sync::RwLock;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom vn1;
        atom err;
        atom notfound;
    }
}

struct DbResource {
    db: RwLock<DB>,
    path: String,
}

rustler_export_nifs!(
    "rocker",
    [
        ("lxcode", 0, lxcode), // library version code
        ("open", 2, open), // open db with options
        ("open_default", 1, open_default), // open db with defaults
        ("destroy", 1 , destroy, NifScheduleFlags::DirtyIo), //destroy db and data
        ("repair", 1 , repair, NifScheduleFlags::DirtyIo), //repair db
        ("path", 1, path), //get fs path
        ("put", 3, put), //put key payload
        ("get", 2, get), //get key payload
        ("delete", 2, get), //delete key
    ],
    Some(on_load)
);

fn on_load<'a>(env: NifEnv<'a>, _load_info: NifTerm<'a>) -> bool {
    resource_struct_init!(DbResource, env);
    true
}

fn lxcode<'a>(env: NifEnv<'a>, _args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    Ok((atoms::ok(), atoms::vn1()).encode(env))
}


fn open<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let path: String = args[0].decode()?;
    let iter: NifMapIterator = args[1].decode()?;

    let mut opts = Options::default();
    for (key, value) in iter {
        let param = try!(key.atom_to_string());
        match param.as_str() {
            "create_if_missing" => {
                if try!(value.atom_to_string()).as_str() == "true" {
                    opts.create_if_missing(true);
                }
            }
            "set_max_open_files" => {
                let limit: i32 = value.decode()?;
                opts.set_max_open_files(limit);
            }
            "set_use_fsync" => {
                if try!(value.atom_to_string()).as_str() == "true" {
                    opts.set_use_fsync(true);
                }
            }
            "set_bytes_per_sync" => {
                let limit: u64 = value.decode()?;
                opts.set_bytes_per_sync(limit);
            }
            "optimize_for_point_lookup" => {
                let limit: u64 = value.decode()?;
                opts.optimize_for_point_lookup(limit);
            }
            "set_table_cache_num_shard_bits" => {
                let limit: i32 = value.decode()?;
                opts.set_table_cache_num_shard_bits(limit);
            }
            "set_max_write_buffer_number" => {
                let limit: i32 = value.decode()?;
                opts.set_max_write_buffer_number(limit);
            }
            "set_write_buffer_size" => {
                let limit: usize = value.decode()?;
                opts.set_write_buffer_size(limit);
            }
            "set_target_file_size_base" => {
                let limit: u64 = value.decode()?;
                opts.set_target_file_size_base(limit);
            }
            "set_min_write_buffer_number_to_merge" => {
                let limit: i32 = value.decode()?;
                opts.set_min_write_buffer_number_to_merge(limit);
            }
            "set_level_zero_stop_writes_trigger" => {
                let limit: i32 = value.decode()?;
                opts.set_level_zero_stop_writes_trigger(limit);
            }
            "set_level_zero_slowdown_writes_trigger" => {
                let limit: i32 = value.decode()?;
                opts.set_level_zero_slowdown_writes_trigger(limit);
            }
            "set_max_background_compactions" => {
                let limit: i32 = value.decode()?;
                opts.set_max_background_compactions(limit);
            }
            "set_max_background_flushes" => {
                let limit: i32 = value.decode()?;
                opts.set_max_background_flushes(limit);
            }
            "set_disable_auto_compactions" => {
                if try!(value.atom_to_string()).as_str() == "true" {
                    opts.set_disable_auto_compactions(true);
                }
            }
            "set_compaction_style" => {
                let style = try!(value.atom_to_string());
                if style == "level" {
                    opts.set_compaction_style(DBCompactionStyle::Level);
                } else if style == "universal" {
                    opts.set_compaction_style(DBCompactionStyle::Universal);
                } else if style == "fifo" {
                    opts.set_compaction_style(DBCompactionStyle::Fifo);
                }
            }
            _ => {}
        }
    }

    let resource = ResourceArc::new(DbResource {
        db: RwLock::new(
            DB::open(&opts, path.clone()).unwrap()
        ),
        path: path.clone(),
    });

    Ok((atoms::ok(), resource.encode(env)).encode(env))
}


fn open_default<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let path: String = args[0].decode()?;

    let resource = ResourceArc::new(DbResource {
        db: RwLock::new(
            DB::open_default(path.clone()).unwrap()
        ),
        path: path.clone(),
    });

    Ok((atoms::ok(), resource.encode(env)).encode(env))
}


fn destroy<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let path: String = args[0].decode()?;
    match DB::destroy(&Options::default(), path) {
        Ok(_) => Ok((atoms::ok()).encode(env)),
        Err(e) => Ok((atoms::err(), e.to_string()).encode(env)),
    }
}


fn repair<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let path: String = args[0].decode()?;
    match DB::repair(Options::default(), path) {
        Ok(_) => Ok((atoms::ok()).encode(env)),
        Err(e) => Ok((atoms::err(), e.to_string()).encode(env)),
    }
}


fn path<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let resource: ResourceArc<DbResource> = args[0].decode()?;
    let path = resource.path.to_string();
    Ok((atoms::ok(), path).encode(env))
}


fn put<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let resource: ResourceArc<DbResource> = args[0].decode()?;
    let key: String = args[1].decode()?;
    let value: String = args[2].decode()?;
    let key_bin: Vec<u8> = key.into_bytes();
    let value_bin: Vec<u8> = value.into_bytes();
    let db = resource.db.write().unwrap();
    match db.put(&key_bin, &value_bin) {
        Ok(_) => Ok((atoms::ok()).encode(env)),
        Err(e) => Ok((atoms::err(), e.to_string()).encode(env)),
    }
}


fn get<'a>(env: NifEnv<'a>, args: &[NifTerm<'a>]) -> NifResult<NifTerm<'a>> {
    let resource: ResourceArc<DbResource> = args[0].decode()?;
    let key: String = args[1].decode()?;
    let key_bin: Vec<u8> = key.into_bytes();
    let db = resource.db.write().unwrap();
    match db.get(&key_bin) {
        Ok(Some(v)) => {
            let res = std::str::from_utf8(&v[..]).unwrap();
            Ok((atoms::ok(), res.to_string()).encode(env))
        }
        Ok(None) => Ok((atoms::notfound()).encode(env)),
        Err(e) => Ok((atoms::err(), e.to_string()).encode(env)),
    }
}