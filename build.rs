#![allow(clippy::multiple_crate_versions, unused)]
use std::env;
use std::fs::read_dir;
use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};

#[cfg(feature = "trusted-service-provider")]
fn generate_ts_bindings(ts_include_dir: String) -> Result<()> {
    let header = ts_include_dir.clone() + "/components/service/locator/interface/service_locator.h";
    let encoding_header = ts_include_dir.clone() + "/protocols/rpc/common/packed-c/encoding.h";

    println!("cargo:rerun-if-changed={}", header);

    let bindings = bindgen::Builder::default()
        .clang_arg(format!(
            "-I{}",
            ts_include_dir + "/components/rpc/common/interface"
        ))
        .rustfmt_bindings(true)
        .header(header)
        .header(encoding_header)
        .generate_comments(false)
        .size_t_is_usize(true)
        .derive_default(true)
        .generate()
        .map_err(|_| {
            Error::new(
                ErrorKind::Other,
                "Unable to generate bindings to trusted services locator",
            )
        })?;
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("ts_bindings.rs"))?;
    // TODO: Remove once we can use the full TS stack and this isn't needed
    println!("cargo:rustc-link-search=native=/usr/lib");
    // libstdc++ is used on Ubuntu 18. On Ubuntu 20, libc++ (from libc++-dev and libc++abi-dev) can be used
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=dylib=ts");
    // TODO: Remove once we can use the full TS stack and this isn't needed
    println!("cargo:rustc-link-lib=static=mbedcrypto");
    println!("cargo:rustc-link-lib=static=protobuf-nanopb");

    Ok(())
}

#[cfg(feature = "trusted-service-provider")]
fn generate_proto_sources(contract_dir: String) -> Result<()> {
    let crypto_pb_dir = contract_dir.clone() + "/service/crypto/protobuf";
    let dir_entries = read_dir(Path::new(&crypto_pb_dir))?;
    let files: Result<Vec<String>> = dir_entries
        .map(|protos_file| {
            protos_file?
                .path()
                .into_os_string()
                .into_string()
                .map_err(|_| {
                    Error::new(
                        ErrorKind::InvalidData,
                        "conversion from OsString to String failed",
                    )
                })
        })
        // Fail the entire operation if there was an error.
        .collect();
    let proto_files: Vec<String> = files?
        .into_iter()
        .filter(|string| string.ends_with(".proto"))
        .collect();
    let files_slices: Vec<&str> = proto_files.iter().map(|file| &file[..]).collect();

    prost_build::compile_protos(&files_slices, &[&contract_dir])
}

#[cfg(feature = "trusted-service-provider")]
fn main() -> Result<()> {
    {
        generate_proto_sources(String::from("trusted-services-vendor/protocols"))?;
        generate_ts_bindings(String::from("trusted-services-vendor"))?;
    }

    Ok(())
}

#[cfg(not(feature = "trusted-service-provider"))]
fn main() {}
