#![feature(collections)]

extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate hyper;


#[macro_use]
extern crate objc;


use std::collections::{VecDeque, VecMap};
use std::option::Option;
use std::any::Any;
use std::boxed::Box;
use std::thread;
use std::io::Read;
use std::sync::{Once, ONCE_INIT};


use libc::{c_void};
use core_foundation::*;
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use cocoa::appkit::{NSEvent};
use cocoa::foundation::{NSString};
use objc::*;
use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::NSAutoreleasePool;

use hyper::Client;
use hyper::header::{Connection, ConnectionOption};
use hyper::status::StatusCode;

mod core_graphics;
mod core_foundation;
mod alut;
mod event_tap;


const QUIT_KEY_SEQ: &'static[u8] = &[12, 0, 6, 18, 19, 20]; //QAZ123
static AUDIO_FILES: [&'static str; 9] = ["1.wav","2.wav","3.wav","4.wav","5.wav","6.wav","7.wav","8.wav", "enter.wav"];
const NON_UNIQ_AUDIO_COUNT:u8 = 8;

fn main() 
{	
	thread::spawn(||
	{
	    let mut client = Client::new();

		//todo: test only
	    let mut result = client.get("http://www.yingdev.com/projects/latestVersion?product=WGestures")
	        .header(Connection(vec![ConnectionOption::Close]))
	        .send();
	    
	    let mut resp;
	    match result
	    {
	    	Ok(mut r) => resp = r,
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
	    		Err(e) => {
	    			println!("Failed to read http content: {}", e);
	    			return;
	    		}
	    	}
	    	println!("Response: {}", content);
	    }else
	    {
	    	println!("Failed to check for update: Status {}", resp.status);
	    }
	    
	});

	unsafe
	{
		alutInit(std::ptr::null_mut(), std::ptr::null_mut());
	}

	let _pool = unsafe{NSAutoreleasePool::new(nil)};

	let mut app = App::new();
	app.load_audio(&find_data_path("bubble"), &AUDIO_FILES);

	app.show_notification("Tickeys正在运行", "按 QAZ123 退出");

	app.run();
}


fn find_data_path(style_name: &str) -> String
{
	let args:Vec<_> = std::env::args().collect();
	let mut data_path = std::path::PathBuf::from(&args[0]);
	data_path.pop();
	data_path.push("data");
	data_path.push(style_name);

	data_path.into_os_string().into_string().unwrap()
}


struct App
{
	audio_data: Vec<AudioData>,
	special_keymap: VecMap<u8>, //keycode -> index

	last_keys: VecDeque<u8>,
	keyboard_monitor: Option< event_tap::KeyboardMonitor>, //defered
	notification_delegate: id
}

impl App
{
	pub fn new() -> App
	{
		unsafe
		{
			let noti_center_del:id = UserNotificationCenterDelegate::new(nil).autorelease();
			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
			let center:id = msg_send![center, setDelegate: noti_center_del];

			let mut app = App{audio_data: Vec::new(), special_keymap: VecMap::new(), last_keys: VecDeque::new(), keyboard_monitor:None, notification_delegate: noti_center_del};
			app
		}

	}

	pub fn run(&mut self)
	{
		let mut tap;

		let ptr_to_self: *mut c_void = unsafe{std::mem::transmute(self)};

		unsafe
		{
			let mut tap_result = event_tap::KeyboardMonitor::new(App::handle_keyboard_event, ptr_to_self);
			match tap_result
			{
				Ok(t) => tap = t,
				Err(msg) => panic!("error: KeyboardMonitor::new: {}", msg)
			}

			let self_:&mut App = std::mem::transmute(ptr_to_self);
			self_.keyboard_monitor = Some(tap);
		}

		
		unsafe
		{
			CFRunLoopRun();
		}
	}

	pub fn stop(&mut self)
	{
		unsafe
		{
			CFRunLoopStop(CFRunLoopGetCurrent());
		}
	}

	pub fn load_audio(&mut self, dir: &str, files: &[&str])
	{
		self.audio_data.clear();

		let mut path = dir.to_string();
		path.push_str("/");
		let base_path_len = path.chars().count();

		for f in files.iter()
		{
			path.push_str(f);
			println!("loading audio:{}", path);
			let audio = AudioData::from_file(&path);

			if audio.source == 0 as ALuint
			{
				panic!("failed to load audio file:{}", f);
			}

			path.truncate(base_path_len);

			self.audio_data.push(audio);
		}
	}

	extern fn handle_keyboard_event(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef
	{
		let keycode = unsafe{CGEventGetIntegerValueField(event, CGEventField::kCGKeyboardEventKeycode)} as u16;

		assert!(refcon != 0 as *mut c_void);

		//todo: temp
		let app: &mut App = unsafe{ std::mem::transmute(refcon)};
		app.handle_keydown(keycode as u8);
		//app.audios[(keycode % 8) as usize].play();
		//play_keycode_audio(keycode);

		event
	}

	fn handle_keydown(&mut self, keycode: u8)
	{	
		self.last_keys.push_back(keycode);
		if self.last_keys.len() > 6 
		{
			self.last_keys.pop_front();
		}

		for i in self.last_keys.iter()
		{
			print!("{} ", i);
		}

		//todo: temp
		if self.last_keys.iter().zip(QUIT_KEY_SEQ.iter()).filter(|&(a,b)| a == b).count() == QUIT_KEY_SEQ.len()
		{
			println!("Quit!");
			self.show_notification("Tickeys", "已退出");
			self.stop();
		}

		println!("key:{}", keycode);



		static mut last_time: u64 = 0;
		static mut last_key: i16 = -1;
		
		let index = match keycode
		{
			36 => 8, //return
			/*51 => 9, //back
			49 => 8, //space
			125 => 6, //down
			126 => 7, //up*/
			_ => (keycode % NON_UNIQ_AUDIO_COUNT) as usize
		};

		unsafe
		{	
			let now = time::precise_time_ns() / 1000 / 1000;

			let delta = now - last_time ;
			println!("interval:{}", delta);
			if delta < 60 && last_key == (keycode as i16)
			{
				last_time = now;
				return;
			}
			last_key = keycode as i16;
			last_time = now;
		}

		self.audio_data[index].play();
	}



	fn show_notification(&mut self, title: &str, msg: &str)
	{
		unsafe
		{
			let note:id = NSUserNotification::new(nil).autorelease();
			note.setTitle_(NSString::alloc(nil).init_str(title));
			note.setInformativeText_(NSString::alloc(nil).init_str(msg));
			
			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];

			msg_send![center, deliverNotification: note]
		}
	}
}



struct AudioData
{
	buffer: ALuint,
	source: ALuint,
    state: ALuint
}

impl AudioData
{
	//todo: how to handle error?
	pub fn from_file(file: &str) -> AudioData
	{
		let file_ptr = std::ffi::CString::new(file).unwrap().as_ptr();
		let mut audio = AudioData{buffer:0, source:0, state:0};
		
		unsafe
		{
			audio.buffer = alutCreateBufferFromFile(file_ptr);
			
			// Create sound source (use buffer to fill source)
    		alGenSources(1, &mut audio.source);
    		alSourcei(audio.source, AL_BUFFER, audio.buffer as ALint);

    		if audio.buffer == 0
    		{
    			panic!("failed to load file: {}", file);
    		}
		}

		audio
	}

	pub fn play(self: &mut AudioData)
	{
		unsafe
		{
			alSourcePlay(self.source);
		}
	}

	pub fn set_pitch(self: &mut AudioData, value: f32)
	{
    	unsafe{alSourcef(self.source, AL_PITCH, value);}
	}

	pub fn set_gain(self: &mut AudioData, value: f32)
	{
		unsafe{alSourcef(self.source, AL_GAIN, value);}
	}
}

impl Drop for AudioData
{
	fn drop(&mut self)
	{
		unsafe
		{
			alDeleteSources(1, &self.source);
    		alDeleteBuffers(1, &self.buffer);
		}
		
	}
}

pub trait NSUserNotification
{
	unsafe fn new(_: Self) -> id
	{
		msg_send![class("NSUserNotification"), new]
	}

	unsafe fn setTitle_(self, title: id);
	unsafe fn setInformativeText_(self, txt: id);
}

impl NSUserNotification for id
{
	unsafe fn setTitle_(self, title: id)
	{
		msg_send![self, setTitle: title]
	}

	unsafe fn setInformativeText_(self, txt: id)
	{
		msg_send![self, setInformativeText: txt]
	}
}


pub trait UserNotificationCenterDelegate //: <NSUserNotificationCenerDelegate>
{
	fn new(_: Self) -> id
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

				let activated_fn: extern fn(&mut Object, Sel, id, id) = Self::userNotificationCenterDidActivateNotification;
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
	
	extern fn userNotificationCenterDidActivateNotification(this: &mut Object, _cmd: Sel, center: id, note: id)
	{
		println!("userNotificationCenterDidActivateNotification");

		unsafe
		{
			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			//todo: extract
			let url:id = msg_send![class("NSURL"), URLWithString: NSString::alloc(nil).init_str("http://www.yingDev.com/projects/Tickeys")];

			let ok:bool = msg_send![workspace, openURL: url];
		}
	}
}

impl UserNotificationCenterDelegate for id
{

}








