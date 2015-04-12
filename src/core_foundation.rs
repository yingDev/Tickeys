#[allow(non_snake_case)]

extern crate libc;
use self::libc::{c_void, c_long};

pub type CFMachPortRef = *mut c_void;//opaque
pub type CFAllocatorRef = *mut c_void;//opaque
pub type CFRunLoopSourceRef = *mut c_void;//opaque
pub type CFRunLoopRef = *mut c_void;//opaque
pub type CFStringRef = *mut c_void;
pub type CFIndex = c_long;

extern
{
	pub static kCFRunLoopDefaultMode: CFStringRef;
	pub static kCFRunLoopCommonModes: CFStringRef;
	pub static kCFAllocatorDefault: CFAllocatorRef;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "system"
{
	pub fn CFMachPortCreateRunLoopSource(allocator: CFAllocatorRef, port: CFMachPortRef, order: CFIndex) -> CFRunLoopSourceRef;
	pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef , mode: CFStringRef);
	pub fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef , mode: CFStringRef);
	pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
	pub fn CFRunLoopRun();
	pub fn CFRunLoopStop(rl: CFRunLoopRef);
}

