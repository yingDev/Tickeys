
extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate hyper;
extern crate block;
extern crate rustc_serialize;
#[macro_use] extern crate objc;
extern crate IOKit_sys as iokit;

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
mod cocoa_util;
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
	unsafe{NSAutoreleasePool::new(nil)};

	request_accessiblility();
	begin_check_for_update(&nsstring_to_string(ns_localized_string("check_update_url")));

	let schems = load_audio_schemes();
	let pref = Pref::load(&schems);

	let mut tickeys = Tickeys::new(schems);
	tickeys.load_scheme(&get_data_path(&pref.audio_scheme), &pref.audio_scheme);
	tickeys.set_volume(pref.volume);
	tickeys.set_pitch(pref.pitch);
	tickeys.set_on_keydown(Some(handle_keydown)); //handle qaz123
	tickeys.start();

	show_notification_nsstring(ns_localized_string("Tickeys_Running"), ns_localized_string("press_qaz123"), noti_click_callback);

	//relaunch on os wakeup from sleep
	register_os_wake_noti();

	//main loop
	app_run();
}


fn register_os_wake_noti()
{
	println!("register_os_wake_noti()");

	#[allow(unused_variables)]
 	extern fn power_callback(ref_con: *mut c_void, service: iokit::io_service_t, message_type: u32, message_argument: *mut c_void)
	{	
		println!("System Power Callback! ");
		match message_type
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
	    let root_port = iokit::IORegisterForSystemPower( ref_con, &mut notify_port_ref as *mut _, power_callback, &mut notifier_object as *mut _);

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


fn request_accessiblility()
{
	println!("request_accessiblility");

	#[link(name = "ApplicationServices", kind = "framework")]
	extern "system"
	{
	 	fn AXIsProcessTrustedWithOptions (options: id) -> bool;
	}

 	unsafe fn is_enabled(prompt: bool) -> bool
 	{
		let dict:id = msg_send![class("NSDictionary"), dictionaryWithObject:(if prompt {kCFBooleanTrue}else{kCFBooleanFalse}) forKey:kAXTrustedCheckOptionPrompt];
		dict.autorelease();
		return AXIsProcessTrustedWithOptions(dict);
	}

	unsafe
	{
		if is_enabled(false) {return;}

		while !is_enabled(true)
		{
			let alert:id = msg_send![class("NSAlert"), new];
			alert.autorelease();
			let _:id = msg_send![alert, setMessageText: ns_localized_string("ax_tip")];
			let _:id = msg_send![alert, addButtonWithTitle: ns_localized_string("quit")];
			let _:id = msg_send![alert, addButtonWithTitle: ns_localized_string("doneWithThis")];

			let btn:i32 = msg_send![alert, runModal];
			println!("request_accessiblility alert: {}", btn);
			match btn
			{
				1001 => {continue},
				1000 => {app_terminate();},
				_ => {panic!("request_accessiblility");}
			}
		}

		app_relaunch_self();
	}
}


fn load_audio_schemes() -> Vec<AudioScheme>
{
	let path = get_res_path("data/schemes.json");
	let mut file = File::open(path).unwrap();

	let mut json_str = String::with_capacity(512);
	match file.read_to_string(&mut json_str)
	{
		Ok(_) => {},
		Err(e) => panic!("Failed to read json:{}",e)
	}

	let schemes:Vec<AudioScheme> = json::decode(&json_str).unwrap();

	schemes
}


fn get_data_path(sub_path: &str) -> String
{
	get_res_path(&("data/".to_string() + sub_path))
}


fn begin_check_for_update(url: &str)
{
	#[derive(RustcDecodable, RustcEncodable)]
	#[allow(non_snake_case)]
	struct Version
	{
		Version: String,
		WhatsNew: String,
	}

	let run_loop_ref = unsafe{CFRunLoopGetCurrent() as usize};
	let check_update_url = url.to_string();
	thread::spawn(move ||
	{
	    let client = Client::new();
	    let result = client.get(&check_update_url).header(Connection::close()).send();

	    let mut resp;
	    match result
	    {
	    	Ok(r) => resp = r,
	    	Err(e) => {
	    		println!("Failed to check for update: {}", e);
	    		return;
	    	}
	    }

	    if resp.status == StatusCode::Ok
	    {
	    	let mut content = String::new();
	    	match resp.read_to_string(&mut content)
	    	{
	    		Ok(_) => {},
	    		Err(e) =>
	    		{
	    			println!("Failed to read http content: {}", e);
	    			return;
	    		}
	    	}
	    	println!("Response: {}", content);

	    	if content.contains("Version")
	    	{
	    		let ver:Version = json::decode(&content).unwrap();
	    		println!("ver={}",ver.Version);
	    		if ver.Version != CURRENT_VERSION
	    		{
	    			let cblock : ConcreteBlock<(),(),_> = ConcreteBlock::new(move ||
			    	{
			    		println!("New Version Available!");
			    		
			    		let title = ns_localized_string("newVersion");
			    		let whats_new = unsafe{NSString::alloc(nil).init_str(&format!("{} -> {}: {}",CURRENT_VERSION, ver.Version, ver.WhatsNew)).autorelease()};
			    	
			    		show_notification_nsstring(title, whats_new, noti_click_callback)
			    	});

			    	let block = & *cblock.copy();

			    	unsafe
			    	{
			    		CFRunLoopPerformBlock(run_loop_ref as *mut c_void, kCFRunLoopDefaultMode, block);
			    	}
		    	}
	    	}
	    }else
	    {
	    	println!("Failed to check for update: Status {}", resp.status);
	    }
	});
}


fn handle_keydown(tickeys: &Tickeys, _:u8)
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
		show_settings(tickeys);
	}
}


fn show_settings(tickeys: &Tickeys)
{
	println!("Settings!");
	unsafe
	{
		SettingsDelegate::get_instance(nil, std::mem::transmute(tickeys));
	}
}


extern fn noti_click_callback(this: &mut Object, _cmd: Sel, center: id, note: id)
{
	println!("noti_click_callback");
	unsafe
	{
		let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
		let url:id = msg_send![class("NSURL"), URLWithString: NSString::alloc(nil).init_str(WEBSITE)];
		let _:bool = msg_send![workspace, openURL: url];

		msg_send![center, removeDeliveredNotification:note]
	}
}





