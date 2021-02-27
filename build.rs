extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::*;

use lazy_static::lazy_static;



lazy_static! {
	static ref OUT_PATH: PathBuf = PathBuf::from( env::var("OUT_DIR").expect("Unable to get output directory") );
}

/// Tries to find all header files in a given include dir
fn add_header_file( bgb: bindgen::Builder, module: &str, include_dir: &Path ) -> bindgen::Builder {

	let header_file_name = "gnunet_".to_owned() + module + ".h";

	// Write config header contents into the new temp file.
	let tmp_file_path = OUT_PATH.join( &header_file_name );
	let mut tmp_file = File::create( &tmp_file_path ).expect("Unable to create new temp file in the output directory.");
	
	// Append include directive to include all configuration macro variables.
	tmp_file.write( "#include \"gnunet_config.h\"\n#include <stdio.h>\n#include <stdbool.h>\n".as_bytes() )
		.expect("Unable to write include statement to temp file.");
	
	// Write actuall header contents into the new temp file.
	let mut header_contents = Vec::new();
	File::open( include_dir.join("gnunet").join( header_file_name ) ).expect("Unable to open header file")
		.read_to_end( &mut header_contents ).expect("Unable to read out header file.");
	tmp_file.write_all( &header_contents ).expect("Unable to write to temp header file.");
	tmp_file.flush().expect("Unable to write to temp header file.");

	bgb.header( tmp_file_path.as_os_str().to_str().unwrap() )
}

fn enable_module( compiler: &mut cc::Build, mut bgb: bindgen::Builder, module: &str, header_name: &str ) -> bindgen::Builder {

	let pkg_name = "gnunet".to_owned() + module;

	match pkg_config::Config::new().atleast_version("0.10").arg("--cflags").probe(&pkg_name) {
		Err(e) => panic!("Unable to find `{}` with pkg-config: {}", module, e),
		Ok( lib ) => {

			for inc in &lib.include_paths {

				bgb = add_header_file( bgb, header_name , inc );

				compiler.include( inc );
				bgb = bgb.clang_arg(format!("-I {}", inc.as_os_str().to_str().unwrap()));
			}
		}
	}

	bgb
}

fn main() {

	// The Docs.rs build system doesn't have gnutls installed.
	if let Ok(_) = env::var("DOCS_RS") {
		File::create( OUT_PATH.join("c_bindings.rs") ).expect("Unable to create dummy file in output path");
		return
	}

	let mut compiler = cc::Build::new();

	let mut include_dir = PathBuf::new();

	// Get the include dirs from `pkg-config`
	match pkg_config::Config::new().atleast_version("0.10").arg("--cflags").probe("gnunetcore") {
		Err(e) => panic!("Unable to find gnunetcore development files: {}", e),
		Ok( lib ) => {

			// Manually add GTK includes to compiler
			for inc in &lib.include_paths {
				fs::copy( inc.join("gnunet/gnunet_config.h"), OUT_PATH.join("gnunet_config.h") ).expect("Unable to copy gnunet_config.h");

				include_dir = inc.clone();
			}

			for lib in &lib.libs {
				println!("cargo:rustc-link-lib={}", lib);
			}
		}
	}
	let mut bgb = bindgen::Builder::default()
		.clang_arg(format!("-I{}", include_dir.as_os_str().to_str().unwrap()));

	bgb = add_header_file( bgb, "ats_service", &include_dir );
	bgb = add_header_file( bgb, "bandwidth_lib", &include_dir );
	bgb = add_header_file( bgb, "bio_lib", &include_dir );
	bgb = add_header_file( bgb, "buffer_lib", &include_dir );
	bgb = add_header_file( bgb, "client_lib", &include_dir );
	bgb = add_header_file( bgb, "common", &include_dir );
	bgb = add_header_file( bgb, "configuration_lib", &include_dir );
	bgb = add_header_file( bgb, "container_lib", &include_dir );
	bgb = add_header_file( bgb, "constants", &include_dir );
	bgb = add_header_file( bgb, "crypto_lib", &include_dir );
	bgb = add_header_file( bgb, "disk_lib", &include_dir );
	bgb = add_header_file( bgb, "dnsstub_lib", &include_dir );
	bgb = add_header_file( bgb, "dnsparser_lib", &include_dir );
	bgb = add_header_file( bgb, "helper_lib", &include_dir );
	bgb = add_header_file( bgb, "hello_lib", &include_dir );
	bgb = add_header_file( bgb, "getopt_lib", &include_dir );
	bgb = add_header_file( bgb, "mst_lib", &include_dir );
	bgb = add_header_file( bgb, "mq_lib", &include_dir );
	bgb = add_header_file( bgb, "nc_lib", &include_dir );
	bgb = add_header_file( bgb, "network_lib", &include_dir );
	bgb = add_header_file( bgb, "nt_lib", &include_dir );
	bgb = add_header_file( bgb, "op_lib", &include_dir );
	bgb = add_header_file( bgb, "os_lib", &include_dir );
	bgb = add_header_file( bgb, "peer_lib", &include_dir );
	bgb = add_header_file( bgb, "plugin_lib", &include_dir );
	bgb = add_header_file( bgb, "program_lib", &include_dir );
	bgb = add_header_file( bgb, "protocols", &include_dir );
	bgb = add_header_file( bgb, "service_lib", &include_dir );
	bgb = add_header_file( bgb, "scheduler_lib", &include_dir );
	bgb = add_header_file( bgb, "signal_lib", &include_dir );
	bgb = add_header_file( bgb, "strings_lib", &include_dir );
	bgb = add_header_file( bgb, "time_lib", &include_dir );
	bgb = add_header_file( bgb, "transport_service", &include_dir );
	bgb = add_header_file( bgb, "tun_lib", &include_dir );
	bgb = add_header_file( bgb, "util_lib", &include_dir );
	bgb = enable_module(&mut compiler, bgb, "core", "core_service");
	bgb = enable_module(&mut compiler, bgb, "util", "util_lib");
	bgb = enable_module(&mut compiler, bgb, "identity", "identity_service");

	if cfg!(feature = "cadet") { bgb = enable_module(&mut compiler, bgb, "cadet", "cadet_service"); }

	if cfg!(feature = "fs")	{ bgb = enable_module(&mut compiler, bgb, "fs", "fs_service"); }

	if cfg!(feature = "peerstore")	{
		bgb = add_header_file(bgb, "peerstore_plugin", &include_dir);
		bgb = add_header_file(bgb, "peerstore_service", &include_dir);
		println!("cargo:rustc-link-lib=gnunetpeerstore");
	}

	// Generate bindings
	bgb.generate().expect("Unable to generate FFI bindings!")
		.write_to_file( OUT_PATH.join("c_bindings.rs") ).expect("Unable to write FFI bindings to file!");
}