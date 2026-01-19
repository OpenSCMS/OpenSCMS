// Copyright (c) 2025 LG Electronics, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::env;

use std::path::{Path, PathBuf};

fn rerun_if_any_file_changed(dir: &Path) {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to get directory entry");
            let path = entry.path();
            if path.is_dir() {
                rerun_if_any_file_changed(&path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}

fn build_oscms() -> (String, String) {
    // Path to the CMake project
    let oscms_codecs_bridge_path = PathBuf::from("oscms-codecs-bridge")
        .canonicalize()
        .expect("cannot canonicalize OSCMS CODECS BRIDGE path");

    // Monitor all files in the oscms-codecs-bridge  and rebuild if any changes
    rerun_if_any_file_changed(&oscms_codecs_bridge_path);

    let install_target = env::var("OSCMS_INSTALL_TARGET_DIR").unwrap_or_else(|_| "".to_string());

    // Determine the correct profile to pass to CMake
    let rust_profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let cmake_profile = match rust_profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        _ => "Debug", // default
    };

    // If OSCMS_INSTALL_TARGET_DIR is definied, use it as the install target. Else use cmake default
    let mut binding = cmake::Config::new(&oscms_codecs_bridge_path);
    let dst = binding.profile(cmake_profile);
    if !install_target.is_empty() {
        dst.define("CMAKE_INSTALL_PREFIX", install_target.clone())
            .define("CMAKE_INSTALL_LIBDIR", "lib")
            .define("CMAKE_INSTALL_INCLUDIR", "include");
    }
    let dst = dst.build();

    // The include path will be `<cpp_lib>/include`
    let include_path = if !install_target.is_empty() {
        PathBuf::from(install_target.clone())
            .join("include")
            .display()
            .to_string()
    } else {
        dst.join("include").display().to_string()
    };

    let lib_path = if !install_target.is_empty() {
        PathBuf::from(install_target)
            .join("lib")
            .display()
            .to_string()
    } else {
        dst.join("lib").display().to_string()
    };

    // Tell cargo where to find the C/C++ library during linking.
    println!("cargo:rustc-link-search={}", lib_path);
    println!("cargo:rustc-link-lib=oscms_bridge");

    (include_path, lib_path)
}

fn main() {
    // Build oscms-bridge C++ library
    let (lib_include_path, _) = build_oscms();
    let cpp_lib_include_path = PathBuf::from(lib_include_path);

    // For bindgen, the header search path can be specified.
    let bindings = bindgen::Builder::default()
        .header(
            cpp_lib_include_path
                .join("oscms_codecs_bridge/wrapper.h")
                .to_str()
                .unwrap(),
        )
        .clang_arg(format!("-I{}", cpp_lib_include_path.to_str().unwrap()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
