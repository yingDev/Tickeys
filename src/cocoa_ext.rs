extern crate objc;
use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use cocoa::appkit::{NSApp,NSApplication};
use std::*;
use std::sync::{ONCE_INIT, Once};

#[allow(non_snake_case)]
#[allow(unused_variables)]
pub trait NSUserNotification
{
	unsafe fn new(_: Self) -> id
	{
		msg_send![class("NSUserNotification"), new]
	}

	unsafe fn setTitle(self, title: id);
	unsafe fn setInformativeText(self, txt: id);
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
impl NSUserNotification for id
{
	unsafe fn setTitle(self, title: id)
	{
		msg_send![self, setTitle: title]
	}

	unsafe fn setInformativeText(self, txt: id)
	{
		msg_send![self, setInformativeText: txt]
	}
}

pub trait RetainRelease
{
	unsafe fn retain(self) -> id;
	unsafe fn release(self) -> id;
}

impl RetainRelease for id
{
	unsafe fn retain(self) -> id
	{
		msg_send![self, retain]
	}
	unsafe fn release(self) -> id
	{
		msg_send![self, release]
	}
}

pub fn ns_localized_string(key: &str) -> id
{
	unsafe
	{
		//[NSBundle mainBundle] localizedStringForKey:(key) value:@"" table:nil]
		let bundle:id = msg_send![class("NSBundle"),mainBundle];
		let s:id = msg_send![bundle, 
						localizedStringForKey:NSString::alloc(nil).init_str(key) 
										value:NSString::alloc(nil).init_str("") 
										table: nil];

		return s;
	}
}

pub fn nsstring_to_string(nsstring: id) -> String 
{
	unsafe
	{
		let len:usize = msg_send![nsstring, length];

		let mut bytes:Vec<u8> = Vec::with_capacity(len);
		bytes.set_len(len);
		ptr::copy_nonoverlapping(nsstring.UTF8String() as *const u8, bytes.as_mut_ptr(), len);
		
		String::from_utf8(bytes).unwrap()
	}
}

pub fn get_res_path(sub_path: &str) -> String
{
	let args:Vec<_> = env::args().collect();
	let mut data_path = path::PathBuf::from(&args[0]);
	data_path.pop();
	data_path.push("../Resources/");
	data_path.push(sub_path);

	data_path.into_os_string().into_string().unwrap()
}

pub fn app_run()
{
	unsafe
	{
		let app = NSApp();
		app.run();
	}
}

pub fn app_relaunch_self()
{
	unsafe
	{
		let bundle:id = msg_send![class("NSBundle"),mainBundle];
		let path:id = msg_send![bundle,  executablePath];

		let proc_info:id = msg_send![class("NSProcessInfo"), processInfo];
		let proc_id:i32 = msg_send![proc_info, processIdentifier];
		let proc_id_str:id = NSString::alloc(nil).init_str(&format!("{}",proc_id)).autorelease();

		let args:id = msg_send![class("NSMutableArray"), new];

		let _:id = msg_send![args, addObject:path];

		let _:id = msg_send![args, addObject:proc_id_str];

		let _:id = msg_send![class("NSTask"), launchedTaskWithLaunchPath:path arguments:args];

	}

	process::exit(0);
}

pub fn app_terminate()
{
	unsafe
	{
		msg_send![NSApp(), terminate:nil]
	}
}



pub fn show_notification_nsstring(title: id, msg: id, activated_fn: extern fn(&mut Object, Sel, id, id))
{
	static REGISTER_DELEGATE: Once = ONCE_INIT;
	REGISTER_DELEGATE.call_once(||
	{
		unsafe
		{
			let noti_center_del:id = UserNotificationCenterDelegate::new(nil,activated_fn).autorelease();
			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
			let _:id = msg_send![center, setDelegate: noti_center_del];
		}
	});

	unsafe
	{
		let note:id = NSUserNotification::new(nil).autorelease();
		note.setTitle(title);
		note.setInformativeText(msg);

		let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];

		msg_send![center, deliverNotification: note]
	}

}

#[allow(non_snake_case)]
#[allow(unused_variables)]
pub trait UserNotificationCenterDelegate //: <NSUserNotificationCenerDelegate>
{
	fn new(_: Self, activated_fn: extern fn(&mut Object, Sel, id, id)) -> id
	{
		static REGISTER_APPDELEGATE: Once = ONCE_INIT;
		REGISTER_APPDELEGATE.call_once(||
		{
			let nsobjcet = objc::runtime::Class::get("NSObject").unwrap();
			let mut decl = objc::declare::ClassDecl::new(nsobjcet, "UserNotificationCenterDelegate").unwrap();

			unsafe
			{
				let delivered_fn: extern fn(&mut Object, Sel, id, id) = Self::userNotificationCenterDidDeliverNotification;
				decl.add_method(sel!(userNotificationCenter:didDeliverNotification:), delivered_fn);

				//let activated_fn: extern fn(&mut Object, Sel, id, id) = Self::userNotificationCenterDidActivateNotification;
				decl.add_method(sel!(userNotificationCenter:didActivateNotification:), activated_fn);
			}

			decl.register();
		});

	    let cls = Class::get("UserNotificationCenterDelegate").unwrap();
	    unsafe
	    {
	        msg_send![cls, new]
    	}
	}

	extern fn userNotificationCenterDidDeliverNotification(this: &mut Object, _cmd: Sel, center: id, note: id)
	{
		println!("userNotificationCenterDidDeliverNotification");
	}

	/*extern fn userNotificationCenterDidActivateNotification(this: &mut Object, _cmd: Sel, center: id, note: id)
	{
		println!("userNotificationCenterDidActivateNotification");

		unsafe
		{
			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			//todo: extract
			let url:id = msg_send![class("NSURL"), URLWithString: NSString::alloc(nil).init_str(WEBSITE)];

			let ok:bool = msg_send![workspace, openURL: url];

			msg_send![center, removeDeliveredNotification:note]
		}
	}*/
}

impl UserNotificationCenterDelegate for id
{

}