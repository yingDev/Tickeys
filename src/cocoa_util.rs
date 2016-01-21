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

pub fn l10n_str(key: &str) -> id
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

pub fn nsstr(from: &str) -> id 
{
	unsafe
	{
		let ns_str = NSString::alloc(nil).init_str(from);
		msg_send![ns_str, autorelease]
	}

}

pub fn nsurl_filename(nsurl: id) -> id
{
	unsafe
	{
		let path_components: id = msg_send![nsurl, pathComponents];
			
		msg_send![path_components, lastObject]
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

pub fn app_run(appDelegate: id)
{
	unsafe
	{
		let app = NSApp();
		let _:id = msg_send![app, setDelegate:appDelegate];
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





/*#[allow(non_snake_case)]
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

}*/



pub fn register_sel(name_with_nul: &str) -> objc::runtime::Sel 
{
	let ptr = name_with_nul.as_ptr() as *const _;
	unsafe { objc::runtime::sel_registerName(ptr) }
}				    	

pub extern fn set_prop<T: objc::Encode>(this: &mut Object, sel: Sel, val: T)
{
	unsafe 
	{
		//set_xxx: -> _xxx
		let mut ivar = sel.name().slice_unchecked(3, sel.name().len() - 1).to_owned();
		let first_char_lower = ivar.remove(0).to_lowercase().next().unwrap();
		ivar.insert(0, first_char_lower);
		ivar.insert(0, '_');

		this.set_ivar::<T>(&ivar, val);
	}
}

pub extern fn get_prop<T: objc::Encode+Copy>(this: &Object, sel: Sel)->T 
{ 
	let mut ivar_name = "_".to_owned();
	ivar_name.push_str(sel.name());
	unsafe { *this.get_ivar::<T>(&ivar_name)} 
}

#[macro_export]
macro_rules! decl_prop 
{
    ($decl: ident, $t: ty, $name: ident) => 
    (
		unsafe
		{
			let mut setter = "set".to_owned();
			let mut name_upper = stringify!($name).to_owned();
			let first_char_upper = name_upper.remove(0).to_uppercase().next().unwrap();
			name_upper.insert(0, first_char_upper);
			name_upper.push(':');
			name_upper.push('\0');
			setter.push_str(&name_upper);

			println!("decl_prop: ivar  ={:}", concat!('_',stringify!($name)));
			println!("decl_prop: getter={:}", stringify!($name));
			println!("decl_prop: setter={:}", &setter);

			$decl.add_ivar::<$t>(concat!('_',stringify!($name)));
			$decl.add_method( register_sel(&setter), set_prop::<$t> as extern fn(&mut Object, Sel, $t));
			$decl.add_method(sel!($name), get_prop::<$t> as extern fn(&Object, Sel)->$t);
		}
    )
}