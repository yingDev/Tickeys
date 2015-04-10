extern crate libc;
use libc::*;
use core_graphics::*;
use core_foundation::*;

pub struct KeyboardMonitor
{
	event_tap: CFMachPortRef,
	runloop_source: CFRunLoopSourceRef,
}

impl KeyboardMonitor
{
	pub unsafe fn new(handler: CGEventTapCallBack, user_data: *mut c_void) -> Result<KeyboardMonitor, String>
	{
		unsafe 
		{
	        let eventTap = CGEventTapCreate(CGEventTapLocation::kCGHIDEventTap, 
						CGEventTapPlacement::kCGHeadInsertEventTap, 
						CGEventTapOptions::kCGEventTapOptionListenOnly,
						CGEventMaskBit!(CGEventType::kCGEventKeyDown),
						handler,
						user_data);

	        if eventTap == (0 as CFMachPortRef)
	        {
	        	return Err("failed to CGEventTapCreate".to_string());
	        }

	        let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0 );
	        if runLoopSource == (0 as CFRunLoopSourceRef)
	        {
	        	return Err("failed to CFMachPortCreateRunLoopSource".to_string());
	        }

	        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource,  kCFRunLoopCommonModes);

	        Ok(KeyboardMonitor{event_tap: eventTap, runloop_source: runLoopSource})
		}
	}
	
	pub fn setEnabled(&mut self, enabled: bool)
	{
		unsafe{CGEventTapEnable(self.event_tap, enabled)};
	}

	pub fn isEnabled(&mut self) -> bool
	{
		unsafe{CGEventTapIsEnabled(self.event_tap)}
	}
}

impl Drop for KeyboardMonitor 
{
	fn drop(&mut self)
	{
		self.setEnabled(false);
		unsafe
		{
			CFRunLoopRemoveSource(CFRunLoopGetCurrent(), self.runloop_source,kCFRunLoopCommonModes);
		}
	}
}