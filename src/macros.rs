//! In this file we reimplement certain functional C macros that didn't get translated automatically.

use crate::bindings::*;

use std::ffi::CString;
use std::os::raw::*;



/// Frees pointers that where allocated by Gnunet
pub unsafe fn GNUNET_free( ptr: *mut c_void ) {
	let cfile = CString::new( file!() ).unwrap();

	GNUNET_xfree_( ptr, cfile.as_ptr(), line!() as _ );
}