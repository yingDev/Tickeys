#[allow(non_camel_case_types)]

extern crate libc;
extern crate block;

use self::libc::{c_void, c_long};
use self::block::Block;

pub type CFTypeRef = *const c_void;
pub type CFMachPortRef = *mut c_void;//opaque
pub type CFAllocatorRef = *mut c_void;//opaque
pub type CFRunLoopSourceRef = *mut c_void;//opaque
pub type CFRunLoopRef = *mut c_void;//opaque
pub type CFStringRef = *mut c_void;
pub type CFBooleanRef = *mut c_void;
pub type CFIndex = c_long;

pub type CFMessagePortRef = *mut c_void;
pub type CFDataRef = *mut c_void;
pub type CFMessagePortCallBack = extern fn (local: CFMessagePortRef, msgid: i32, data: CFDataRef, info: *mut c_void) -> CFDataRef;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[repr(C)]
struct CFMessagePortContext
{
	version: CFIndex,
	info: *mut c_void,
	retain: fn(info: *const c_void) -> *const c_void,
	release: fn(info: *const c_void),
	copyDescription: fn(info: *const c_void) -> CFStringRef
}

extern
{
	pub static kCFRunLoopDefaultMode: CFStringRef;
	pub static kCFRunLoopCommonModes: CFStringRef;
	pub static kCFAllocatorDefault: CFAllocatorRef;

	pub static kCFBooleanTrue: CFBooleanRef;
    pub static kCFBooleanFalse: CFBooleanRef;
    pub static kAXTrustedCheckOptionPrompt: CFStringRef;

}

#[allow(dead_code)]
#[link(name = "CoreFoundation", kind = "framework")]
extern "system"
{
	pub fn CFRelease (cf: CFTypeRef); 

	pub fn CFMachPortCreateRunLoopSource(allocator: CFAllocatorRef, port: CFMachPortRef, order: CFIndex) -> CFRunLoopSourceRef;
	pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef , mode: CFStringRef);
	pub fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef , mode: CFStringRef);
	pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
	pub fn CFRunLoopRun();
	pub fn CFRunLoopStop(rl: CFRunLoopRef);


	pub fn CFRunLoopPerformBlock(rl: CFRunLoopRef,mode: CFTypeRef, block: &Block<(),()> ); 

	pub fn CFMessagePortCreateLocal(allocator: CFAllocatorRef, 
									name: CFStringRef, 
									callout: CFMessagePortCallBack, 
									context: *mut CFMessagePortContext, 
									shouldFreeInfo: bool) -> CFMessagePortRef;
	pub fn CFMessagePortCreateRunLoopSource(allocator: CFAllocatorRef, local: CFMessagePortRef, order: CFIndex) -> CFRunLoopSourceRef;
	pub fn CFMessagePortSendRequest(remote: CFMessagePortRef, 
									msgid: i32, 
									data: CFDataRef, 
									sendTimeout: f64, 
									rcvTimeout: f64, 
									replyMode: CFStringRef,
									returnData: *mut CFDataRef) -> i32;
	pub fn CFMessagePortInvalidate(ms: CFMessagePortRef);

}

