
extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate hyper;
extern crate block;
extern crate rustc_serialize;
#[macro_use] extern crate objc;
extern crate IOKit_sys as iokit;

use std::sync::{ONCE_INIT, Once};
use std::thread;
use std::io::Read;
use std::string::String;
use std::fs::File;
use libc::{c_void};
use core_foundation::*;
use objc::*;
use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use cocoa::appkit::{NSApp,NSApplication};
use hyper::Client;
use hyper::header::{Connection};
use hyper::status::StatusCode;
use self::block::{ConcreteBlock};
use rustc_serialize::json;

mod core_graphics;
mod core_foundation;
mod alut;
mod event_tap;
mod tickeys;
#[macro_use] mod cocoa_util;
mod consts;
mod settings_ui;
mod pref;

use tickeys::*;
use cocoa_util::*;
use settings_ui::*;
use consts::*;
use pref::*;


fn main()
{
	unsafe { NSAutoreleasePool::new(nil); }

	monitor_os_power_event();
	
	let appDelegate = <id as AppDelegate>::new();
	app_run(appDelegate);
}

fn monitor_os_power_event()
{
	println!("monitor_os_power_event()");
	#[allow(unused_variables)]
 	extern fn power_callback(ref_con: *mut c_void, service: iokit::io_service_t,
		msg: u32, msg_args: *mut c_void)
	{
		println!("System Power Callback! ");
		match msg
		{
			iokit::kIOMessageSystemHasPoweredOn =>
			{
				println!("System PoweredOn");
				app_relaunch_self(); //just relaunch;
			},
			_ => {}
		}
	}

	unsafe
	{
		// notification port allocated by IORegisterForSystemPower
	    let mut notify_port_ref: iokit::IONotificationPortRef = std::ptr::null_mut();
	    // notifier object, used to deregister later
	    let mut notifier_object: iokit::io_object_t = 0;
	    // this parameter is passed to the callback
	    let ref_con: *mut c_void = std::ptr::null_mut();
	    // register to receive system sleep notifications
	    let root_port = iokit::IORegisterForSystemPower( ref_con, &mut notify_port_ref as *mut _,
			power_callback, &mut notifier_object as *mut _);

	    if root_port == 0
	    {
	        println!("IORegisterForSystemPower failed\n");
	        return; //ignore for now
	    }
	    // add the notification port to the application runloop
	    core_foundation::CFRunLoopAddSource( core_foundation::CFRunLoopGetCurrent(),
	    	iokit::IONotificationPortGetRunLoopSource(notify_port_ref) as CFRunLoopSourceRef,
	    	core_foundation::kCFRunLoopCommonModes );
	}
}

trait AppDelegate
{
	fn new() -> id
	{
		static REG_OBJC_CLS: Once = ONCE_INIT;
		REG_OBJC_CLS.call_once(||
		{
			let nsobjcet = objc::runtime::Class::get("NSObject").unwrap();
			let mut decl = objc::declare::ClassDecl::new(nsobjcet, "AppDelegate").unwrap();

			unsafe
			{
				let finish_launch_fn: extern fn(&mut Object, Sel, id) = Self::applicationDidFinishLaunching;
				decl.add_method(sel!(applicationDidFinishLaunching:), finish_launch_fn);

				let activated_fn: extern fn(&mut Object, Sel, id) = Self::applicationDidBecomeActive;
				decl.add_method(sel!(applicationDidBecomeActive:), activated_fn);

				let will_term_fn: extern fn(&mut Object, Sel, id) = Self::applicationWillTerminate;
				decl.add_method(sel!(applicationWillTerminate:), will_term_fn);

				let activated_fn: extern fn(&mut Object, Sel, id, id) = Self::userNotificationCenterDidActivateNotification;
				decl.add_method(sel!(userNotificationCenter:didActivateNotification:), activated_fn);

				decl_prop!(decl, usize, tickeys);
			}

			decl.register();
		});

	    unsafe { msg_send![class("AppDelegate"), new] }
	}

	extern fn applicationDidFinishLaunching(this: &mut Object, _cmd: Sel, note: id)
	{
		Self::request_ax();
		Self::begin_check_update(this, &nsstring_to_string(l10n_str("check_update_url")));

		let sch = Self::load_schemes();
		let pref = Pref::load(&sch);
		let mut tickeys = Box::new(Tickeys::new(sch));
		tickeys.load_scheme(&get_res_path(&format!("data/{:}", &pref.scheme)), &pref.scheme);
		tickeys.set_volume(pref.volume);
		tickeys.set_pitch(pref.pitch);
		tickeys.set_on_keydown(Some(Self::handle_keydown)); //handle qaz123
		tickeys.start();
		
		unsafe
		{
			let _: id = msg_send![this, setTickeys: tickeys]; //moved

			let noti_center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
			let _:id = msg_send![noti_center, setDelegate: this];
		}

		Self::show_noti(l10n_str("Tickeys_Running"), l10n_str("press_qaz123"));
	}

	extern fn applicationDidBecomeActive(this: &mut Object, _cmd: Sel, note: id)
	{
		println!("applicationDidBecomeActive");
		Self::show_settings(this);
	}

	extern fn applicationWillTerminate(this: &mut Object, _cmd: Sel, note: id)
	{
		//let it drop
		let tickeys: Box<Tickeys> = unsafe { msg_send![this, tickeys] };
	}

	extern fn userNotificationCenterDidActivateNotification(this: &mut Object, _cmd: Sel, center: id, note: id)
	{
		println!("userNotificationCenterDidActivateNotification");

		unsafe
		{
			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			let url:id = msg_send![class("NSURL"), URLWithString: NSString::alloc(nil).init_str(WEBSITE)];
			let _:bool = msg_send![workspace, openURL: url];

			msg_send![center, removeDeliveredNotification:note]
		}
	}

	fn show_noti(title: id, msg: id)
	{
		unsafe
		{
			let note:id = NSUserNotification::new(nil).autorelease();
			note.setTitle(title);
			note.setInformativeText(msg);

			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];

			msg_send![center, deliverNotification: note]
		}
	}

	fn handle_keydown(tickeys: &Tickeys, key: u8)
	{
		let last_keys = tickeys.get_last_keys();
		let last_keys_len = last_keys.len();

		let mut pass = false;
		for seq in OPEN_SETTINGS_KEY_SEQ
		{
			let seq_len = seq.len();
			if last_keys_len < seq_len {return;}

			pass = true;
			//cmp from tail to head
			for i in 1..(seq_len+1)
			{
				if last_keys[last_keys_len - i] != seq[seq_len - i]
				{
					pass = false;
					break;
				}
			}

			if pass { break;}
		}

		if pass
		{
			Self::show_settings( unsafe{ msg_send![NSApp(), delegate] } );
		}
	}

	fn begin_check_update(this: &mut Object, url: &str)
	{
		#[derive(RustcDecodable, RustcEncodable)]
		#[allow(non_snake_case)]
		struct Version
		{
			Version: String,
			WhatsNew: String,
		}

		let run_loop_ref = unsafe { CFRunLoopGetCurrent() as usize };
		let check_update_url = url.to_string();
		let ptr_to_this: usize = unsafe { std::mem::transmute(this) };
		thread::spawn(move ||
		{
			thread::sleep_ms(1000 * 30); //do it xx seconds later.
			println!("begin_check_update do_job!");
			match do_job(ptr_to_this, check_update_url, run_loop_ref)
			{
				Ok(()) => println!("begin_check_update(): Ok"),
				Err(e) => println!("begin_check_update() Error: {:}", e)
			}
		});

		fn do_job(this: usize, check_update_url: String, run_loop_ref: usize) -> Result<(), hyper::Error>
		{
			let client = Client::new();
		    let mut resp = try!{ client.get(&check_update_url).header(Connection::close()).send() };
		    if resp.status == StatusCode::Ok
		    {
		    	let mut content = String::new();
				try!{ resp.read_to_string(&mut content) };
		    	println!("Response: {}", content);

		    	if content.contains("Version")
		    	{
		    		let ver:Version = json::decode(&content).unwrap();
		    		println!("ver={}",ver.Version);
		    		if ver.Version != CURRENT_VERSION
		    		{
		    			let cblock : ConcreteBlock<(),(),_> = ConcreteBlock::new(move ||
				    	{
				    		let this_ptr: &mut Object = unsafe{ std::mem::transmute(this) };
				    		<id as AppDelegate>::handle_update_info(this_ptr, ver.Version.clone(), ver.WhatsNew.clone());
				    	});

				    	let block = & *cblock.copy();
				    	unsafe { CFRunLoopPerformBlock(run_loop_ref as *mut c_void, kCFRunLoopDefaultMode, block); }
			    	}
		    	}
				return Ok(());
		    }else
		    {
		    	println!("Failed to check for update: Status {}", resp.status);
				return Err(hyper::Error::Status);
		    }
		}
	}

	fn handle_update_info(this: &mut Object, ver: String, whatsNew: String)
	{
	    println!("New Version Available!");
		let title = l10n_str("newVersion");
		let whats_new = unsafe
		{
			NSString::alloc(nil).init_str(
				&format!("{} -> {}: {}",CURRENT_VERSION, ver, whatsNew)
			).autorelease()
		};
		Self::show_noti(title, whats_new);
	}

	fn request_ax()
	{
		println!("request_ax");
		#[link(name = "ApplicationServices", kind = "framework")]
		extern "system"
		{
		 	fn AXIsProcessTrustedWithOptions (options: id) -> bool;
		}

	 	unsafe fn is_enabled(prompt: bool) -> bool
	 	{
			let dict: id = msg_send![class("NSDictionary"),
				dictionaryWithObject: (if prompt {kCFBooleanTrue}else{kCFBooleanFalse})
				forKey: kAXTrustedCheckOptionPrompt];

			return AXIsProcessTrustedWithOptions(dict);
		}

		unsafe
		{
			if is_enabled(false) { return; }
			while !is_enabled(true)
			{
				thread::sleep_ms(3000);

				let alert:id = msg_send![class("NSAlert"), new];
				alert.autorelease();
				let _:id = msg_send![alert, setMessageText: l10n_str("ax_tip")];
				let _:id = msg_send![alert, addButtonWithTitle: l10n_str("quit")];
				let _:id = msg_send![alert, addButtonWithTitle: l10n_str("doneWithThis")];

				let btn:i32 = msg_send![alert, runModal];
				println!("request_ax alert: {}", btn);
				match btn
				{
					1001 => continue,
					1000 => app_terminate(),
					_ => panic!("request_ax")
				}
			}

			app_relaunch_self();
		}
	}

	fn show_settings(this: &mut Object)
	{
		println!("Settings!");
		unsafe
		{
			let tickeys: Box<Tickeys> =  msg_send![this, tickeys];
			SettingsDelegate::get_instance(nil, std::mem::transmute(tickeys));
		}
	}

	fn load_schemes() -> Vec<AudioScheme>
	{
		let path = get_res_path("data/schemes.json");
		let mut file = File::open(path).unwrap();

		let mut json_str = String::with_capacity(512);
		match file.read_to_string(&mut json_str)
		{
			Ok(_) => {},
			Err(e) => panic!("Failed to read json:{}",e)
		}
		json::decode(&json_str).unwrap()
	}
}

impl AppDelegate for id
{}
