
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

extern 
{ 
	static NSWorkspaceDidActivateApplicationNotification: id;
	static NSWorkspaceApplicationKey: id;
	static NSKeyValueChangeNewKey: id;

}

static mut RUNNING: bool = false; 


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

#[repr(i32)]
enum FilterListMode 
{
	BlackList = 0,
	WhiteList = 1
}

trait AppDelegate // <NSApplicationDelegate>
{
	fn new() -> id
	{
		static REG_OBJC_CLS: Once = ONCE_INIT;
		REG_OBJC_CLS.call_once(||
		{
			let nsobjcet = objc::runtime::Class::get("NSObject").unwrap();
			let mut decl = objc::declare::ClassDecl::new(nsobjcet, stringify!(AppDelegate)).unwrap();

			unsafe
			{
				decl.add_method(sel!(applicationDidFinishLaunching:), Self::applicationDidFinishLaunching as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(applicationDidBecomeActive:), Self::applicationDidBecomeActive as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(applicationWillTerminate:), Self::applicationWillTerminate as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(userNotificationCenter:didActivateNotification:), Self::userNotificationCenterDidActivateNotification as extern fn(&mut Object, Sel, id, id));
				decl.add_method(sel!(workspace_app_activated:), Self::workspace_app_activated as extern fn(&mut Object, Sel, id));

				decl.add_method(sel!(observeValueForKeyPath:ofObject:change:context:), 
						Self::observeValueForKeyPathOfObjectChangeContext as extern fn(&mut Object, Sel, id, id, id, *const c_void));

				decl_prop!(decl, usize, tickeys);
				decl_prop!(decl, id, filterList);
				decl_prop!(decl, i32, filterListMode);
			}

			decl.register();
		});

	    unsafe 
	    { 
	    	let inst: id = msg_send![class(stringify!(AppDelegate)), new];

	    	let userDefaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];

	    	let mut filterList: id = msg_send![userDefaults, objectForKey: nsstr("FilterList")];
	    	if filterList == nil 
	    	{
	    		filterList = msg_send![class("NSMutableArray"), arrayWithCapacity: 8];
	    	}else 
	    	{
	    		filterList = msg_send![class("NSMutableArray"), arrayWithArray: filterList];
	    	}
	    	let _: id = msg_send![inst, setFilterList: filterList];


	    	//get filter mode
	    	let filterListMode: i32 = msg_send![userDefaults, integerForKey: nsstr("FilterListMode")];
	    	let _: id = msg_send![inst, setFilterListMode: filterListMode];
	    	println!("FilterListMode = {:}", filterListMode);

	    	inst
	    }
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
			let _:id = msg_send![noti_center, setDelegate: this as *mut Object];
		}

		Self::show_noti(l10n_str("Tickeys_Running"), l10n_str("press_qaz123"));

		unsafe
		{
			//observe NSWorkspaceDidActivateApplicationNotification
			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			let notiCenter: id = msg_send![workspace, notificationCenter];

			let _: id = msg_send![notiCenter, addObserver:this as *mut Object
										    	selector:sel!(workspace_app_activated:) 
											  	    name:NSWorkspaceDidActivateApplicationNotification 
												  object:nil];

			//observe FilterListMode 
			let ud: id = msg_send![class("NSUserDefaultsController"), sharedUserDefaultsController];
			let _: id = msg_send![ud, addObserver:this as *mut Object
                                       forKeyPath:nsstr("values.FilterListMode")
                                          options:1 /*NSKeyValueObservingOptionNew*/
                                          context:0];

             
            //get current active app 
	    	let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
	    	let frontApp: id = msg_send![workspace, frontmostApplication];
			Self::check_and_apply_mute_for_app(this, nsurl_filename(msg_send![frontApp, bundleURL]));

		}

	}

	extern fn applicationDidBecomeActive(this: &mut Object, _cmd: Sel, note: id)
	{
		unsafe 
		{
			if !RUNNING { return; }
		}

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

	extern fn workspace_app_activated(this: &mut Object, cmd: Sel, noti: id)
	{
		unsafe 
		{
			let dict: id = msg_send![noti, userInfo];
			let app: id = msg_send![dict, objectForKey: NSWorkspaceApplicationKey];

			let app_url: id = msg_send![app, bundleURL];
			let path_components: id = msg_send![app_url, pathComponents];
			
			let app_name: id = msg_send![path_components, lastObject];

			Self::check_and_apply_mute_for_app(this, app_name);
		}
	}

	fn check_and_apply_mute_for_app(this: &Object, app_name: id)
	{
		unsafe
		{
			let filterList: id = msg_send![this, filterList];

			////=========
			let isInList: bool = msg_send![filterList, containsObject: app_name];
			let filterListMode: i32 = msg_send![this, filterListMode];

			println!("filterlistmode = {:?}", filterListMode);
			let shouldMute = match filterListMode
			{
				0 => isInList,
				1 => !isInList,
				_ => false,
			};

			let mut tickeys: Box<Tickeys> = msg_send![this, tickeys];
			tickeys.set_mute(shouldMute);
			std::mem::forget(tickeys);

			//println!("workspace_app_activated: {:}, shouldMute: {:}", nsstring_to_string(app_name), shouldMute);
		}
			
	}

	//- (void)observeValueForKeyPath:(NSString *)keyPath ofObject:(id)object change:(NSDictionary *)change context:(void *)context
	extern fn observeValueForKeyPathOfObjectChangeContext(this: &mut Object, cmd: Sel, keypath: id, object: id, change: id, context: *const c_void)
	{
		println!("FilterListMode Changed!");

		unsafe
		{
			if context == (0 as *const c_void)
			{
				let ud: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
				let newValue: i32 = msg_send![ud, integerForKey: nsstr("FilterListMode")];

				let _: id = msg_send![this, setFilterListMode: newValue];

				println!("FilterListMode Changed! {:}", newValue);

			}else 
			{
				//... super call ?
			}
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
			if is_enabled(false) 
			{ 
				RUNNING = true;
				return; 
			}
			
			while !is_enabled(true)
			{
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
			SettingsController::get_instance(nil, std::mem::transmute(tickeys));
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
