
extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate hyper;
extern crate toml;
extern crate block;


#[macro_use]
extern crate objc;

use std::collections::{VecDeque, HashMap};
use std::option::Option;
use std::any::Any;
use std::boxed::Box;
use std::thread;
use std::io::Read;
use std::sync::{Once, ONCE_INIT};
use std::string::String;
use std::fs::File;

use libc::{c_void};
use core_foundation::*;
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use objc::*;
use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::{NSUInteger, NSRect, NSPoint, NSSize,
						NSAutoreleasePool, NSProcessInfo, NSString};
use cocoa::appkit::{NSApp,
					NSApplication, NSApplicationActivationPolicyRegular,
					NSWindow, NSTitledWindowMask, NSBackingStoreBuffered,
					NSMenu, NSMenuItem};

use hyper::Client;
use hyper::header::{Connection, ConnectionOption};
use hyper::status::StatusCode;

use self::block::{Block, ConcreteBlock};

//自己的modules才需要声明
mod core_graphics;
mod core_foundation;
mod alut;
mod event_tap;

const CURRENT_VERSION : &'static str = "0.1.0";

const QUIT_KEY_SEQ: &'static[u8] = &[12, 0, 6, 18, 19, 20]; //QAZ123


//todo: 暂时写死。以后应该从配置加载。
static BUBBLE_AUDIO_FILES: [&'static str; 9] = ["1.wav","2.wav","3.wav","4.wav","5.wav","6.wav","7.wav","8.wav", "enter.wav"];
const BUBBLE_NON_UNIQ_AUDIO_COUNT:u8 = 8;

static TYPEWRITER_AUDIO_FILES: [&'static str; 10] = ["key-new-05.wav","key-new-04.wav","key-new-03.wav","key-new-02.wav","key-new-01.wav","space-new.wav","scrollUp.wav","scrollDown.wav","backspace.wav", "return-new.wav"];
const TYPEWRITER_NON_UNIQ_AUDIO_COUNT:u8 = 5;

static MECHANICAL_AUDIO_FILES: [&'static str; 5] = ["1.wav", "2.wav", "3.wav", "4.wav", "5.wav"];
const MECHANICAL_NON_UNIQ_AUDIO_COUNT:u8 = 4;

fn main() 
{	
	let app_cfg = load_app_config();

	let pool = unsafe{NSAutoreleasePool::new(nil)};

	let mut app = App::new();

	let pref = Pref::load();

	//todo: dirty
	match pref.audio_scheme.as_ref()
	{
		"bubble" => 
		{
			app.load_audio(&(get_data_path("bubble")), &BUBBLE_AUDIO_FILES);
			let mut keymap:HashMap<u8, u8> = HashMap::new();
			keymap.insert(36, 8);
			app.set_keymap(keymap, BUBBLE_NON_UNIQ_AUDIO_COUNT);
		},
		"typewriter" => 
		{
			app.load_audio(&(get_data_path("typewriter")), &TYPEWRITER_AUDIO_FILES);
			let mut keymap:HashMap<u8, u8> = HashMap::new();
			keymap.insert(36, 9);
			keymap.insert(49, 5);
			keymap.insert(51, 8);
			app.set_keymap(keymap, BUBBLE_NON_UNIQ_AUDIO_COUNT);
		},
		"mechanical" => 
		{
			app.load_audio(&(get_data_path("mechanical")), &MECHANICAL_AUDIO_FILES);
			let mut keymap:HashMap<u8, u8> = HashMap::new();
			keymap.insert(36, 4);
			app.set_keymap(keymap, MECHANICAL_NON_UNIQ_AUDIO_COUNT);
		},
		_ => panic!("unrecognized scheme:{}", pref.audio_scheme)
	}
	app.set_volume(pref.volume);
	app.set_pitch(pref.pitch);



	App::show_notification("Tickeys正在运行", "按 QAZ123 打开设置");
	
	app.check_for_update(app_cfg.lookup("config.check_update_api").unwrap().as_str().unwrap());

	app.run();
}

fn load_app_config() -> toml::Value
{
	let mut cfg_path = get_data_path("app_config.toml");

	let mut toml_file = match File::open(cfg_path.clone())
	{
		Ok(f) => f, 
		Err(e) => panic!("Error open file:{} : {}", e, cfg_path)
	};
	let mut toml_str = String::new();
	let n_read = toml_file.read_to_string(&mut toml_str);
	match n_read
	{
		Ok(_) => {},
		Err(e) => panic!("Failed Reading file content:{}", e)
	};
 
	toml_str.parse().unwrap()
}


fn get_data_path(sub_path: &str) -> String
{
	let args:Vec<_> = std::env::args().collect();
	let mut data_path = std::path::PathBuf::from(&args[0]);
	data_path.pop();
	data_path.push("data");
	data_path.push(sub_path);

	data_path.into_os_string().into_string().unwrap()
}


struct Pref
{
	audio_scheme: String,
	volume: f32,
	pitch: f32
}

impl Pref
{
	fn load() -> Pref
	{
		unsafe
		{		
			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			let pref_exists_key:id = NSString::alloc(nil).init_str("pref_exists");
				
			let pref = Pref{audio_scheme: "bubble".to_string(), volume: 0.5f32, pitch: 1.0f32};

			let pref_exists: id = msg_send![user_defaults, stringForKey: pref_exists_key];
			if pref_exists == nil //first run 
			{
				pref.save();
				return pref;
			}else
			{
				let audio_scheme: id = msg_send![user_defaults, stringForKey:NSString::alloc(nil).init_str("audio_scheme")];
				let volume: f32 = msg_send![user_defaults, floatForKey: NSString::alloc(nil).init_str("volume")];
				let pitch: f32 = msg_send![user_defaults, floatForKey: NSString::alloc(nil).init_str("pitch")];
				
				let len:usize = msg_send![audio_scheme, length];
				
				let mut scheme_bytes:Vec<u8> = Vec::with_capacity(len);
        		scheme_bytes.set_len(len);
       			std::ptr::copy_nonoverlapping(audio_scheme.UTF8String() as *const u8, scheme_bytes.as_mut_ptr(), len);
				let scheme_str = String::from_utf8(scheme_bytes).unwrap();
				
				Pref{audio_scheme:  scheme_str, volume: volume, pitch: pitch}
			}

		}

	}

	fn save(&self)
	{
		unsafe
		{
			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];

			let _:id = msg_send![user_defaults, setObject: NSString::alloc(nil).init_str(&self.audio_scheme) forKey: NSString::alloc(nil).init_str("audio_scheme")];
			let _:id = msg_send![user_defaults, setFloat: self.volume forKey: NSString::alloc(nil).init_str("volume")];
			let _:id = msg_send![user_defaults, setFloat: self.pitch forKey: NSString::alloc(nil).init_str("pitch")];

			let pref_exists_key:id = NSString::alloc(nil).init_str("pref_exists");
			let _:id = msg_send![user_defaults, setObject:pref_exists_key forKey: pref_exists_key];

			let _:id = msg_send![user_defaults, synchronize];
		}


	}
}


struct App
{
	volume:f32,
	pitch:f32,

	audio_data: Vec<AudioData>,
	keymap: HashMap<u8, u8>,
	first_n_non_unique: i16,

	last_keys: VecDeque<u8>,
	keyboard_monitor: Option< event_tap::KeyboardMonitor>, //defered
	notification_delegate: id,
	showing_gui:bool
}

impl App
{
	pub fn new() -> App
	{
		unsafe
		{
			alutInit(std::ptr::null_mut(), std::ptr::null_mut());
		}

		unsafe
		{
			let noti_center_del:id = UserNotificationCenterDelegate::new(nil).autorelease();
			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
			let center:id = msg_send![center, setDelegate: noti_center_del];

			let mut app = App{
				volume:1f32,
				pitch:1f32, 
				audio_data: Vec::new(), 
				keymap: HashMap::new(),
				first_n_non_unique: -1,
				last_keys: VecDeque::new(), 
				keyboard_monitor:None, 
				notification_delegate: noti_center_del,
				//settings_delegate: settings_delegate,
				showing_gui: false
			};
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
			let app = NSApp();
			app.run();

		}
	}

	pub fn stop(&mut self)
	{
		unsafe
		{
			//self.settings_delegate.release();
			msg_send![NSApp(), terminate:nil]
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
			let mut audio = AudioData::from_file(&path);

			if audio.source == 0 as ALuint
			{
				panic!("failed to load audio file:{}", f);
			}

			path.truncate(base_path_len);

			audio.set_gain(self.volume);
			audio.set_pitch(self.pitch);

			self.audio_data.push(audio);
		}

		self.keymap.clear();
		self.first_n_non_unique = self.audio_data.len() as i16;
	}

	pub fn set_volume(&mut self, volume: f32)
	{
		//todo:
		self.volume = volume;
		for audio in self.audio_data.iter_mut()
		{
			audio.set_gain(volume);
		}
	}

	pub fn set_pitch(&mut self, pitch: f32)
	{
		//todo:
		self.pitch = pitch;
		for audio in self.audio_data.iter_mut()
		{
			audio.set_pitch(pitch);
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

		if self.last_keys.iter().zip(QUIT_KEY_SEQ.iter()).filter(|&(a,b)| a == b).count() == QUIT_KEY_SEQ.len()
		{
			self.show_settings();
		}

		println!("key:{}", keycode);


		static mut last_time: u64 = 0;
		static mut last_key: i16 = -1;

		let index:i32 = match self.keymap.get(&keycode)
		{
			Some(idx) => *idx as i32,
			None => 
			{
				if self.first_n_non_unique <= 0 
				{
					-1
				}else
				{
					(keycode % (self.first_n_non_unique as u8)) as i32
				}
			}
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

		if index == -1 
		{
			return;
		}

		let audio = &mut self.audio_data[index as usize];
		//audio.set_gain(self.volume);
		//audio.set_pitch(self.pitch);
		audio.play();
	}

	fn set_keymap(&mut self, keymap: HashMap<u8, u8>, first_n_non_unique: u8)
	{
		self.keymap = keymap;
		self.first_n_non_unique = first_n_non_unique as i16;
	}

	fn show_notification(title: &str, msg: &str)
	{
		unsafe
		{
			let note:id = NSUserNotification::new(nil).autorelease();
			note.setTitle(NSString::alloc(nil).init_str(title));
			note.setInformativeText(NSString::alloc(nil).init_str(msg));
			
			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];

			msg_send![center, deliverNotification: note]
		}
	}

	fn show_settings(&mut self)
	{
		println!("Settings!");

		if self.showing_gui
		{
			return;
		}
		self.showing_gui = true;
					
		let settings_delegate = SettingsDelegate::new(nil, self);

	}

	fn check_for_update(&self, url: &str)
	{
		let runloopRef = unsafe{CFRunLoopGetCurrent() as usize};

		let mut check_update_url = String::new();
		check_update_url.push_str(url);

		thread::spawn(move ||
		{
		    let mut client = Client::new();

			//todo: test only
		    let mut result = client.get(&check_update_url)
		        .header(Connection::close())
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

		    	if content != CURRENT_VERSION
		    	{
		    		let ver = content.clone();
			    	let cblock : ConcreteBlock<(),(),_> = ConcreteBlock::new(move ||
			    	{
			    		println!("New Version Available!");
			    		let info_str = format!("{} -> {}", CURRENT_VERSION, ver);
			    		App::show_notification("新版本可用!", &info_str);
			    	});
			    	
			    	let mut block = &mut *cblock.copy();

			    	unsafe
			    	{
			    		CFRunLoopPerformBlock(runloopRef as *mut c_void, kCFRunLoopDefaultMode, block);
			    	}
		    	}


		    }else
		    {
		    	println!("Failed to check for update: Status {}", resp.status);
		    }
		});
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

	unsafe fn setTitle(self, title: id);
	unsafe fn setInformativeText(self, txt: id);
}

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

			msg_send![center, removeDeliveredNotification:note]
		}
	}
}

impl UserNotificationCenterDelegate for id
{

}

trait SettingsDelegate
{
	fn new(_:Self, ptr_to_app: *mut App) -> id
	{
		static REGISTER_APPDELEGATE: Once = ONCE_INIT;
		REGISTER_APPDELEGATE.call_once(||
		{
			println!("SettingsDelegate::new::REGISTER_APPDELEGATE");
			let nsobjcet = objc::runtime::Class::get("NSObject").unwrap();
			let mut decl = objc::declare::ClassDecl::new(nsobjcet, "SettingsDelegate").unwrap();

			unsafe
			{
				//property ptr_to_app
				decl.add_ivar::<usize>("_user_data");
				let set_user_data_fn: extern fn(&mut Object, Sel, usize) = Self::set_user_data_;
				decl.add_method(sel!(setUser_data:), set_user_data_fn);

				let get_user_data_fn: extern fn(&Object, Sel)->usize = Self::get_user_data_;
				decl.add_method(sel!(user_data), get_user_data_fn);

				//property popup_audio_scheme
				decl.add_ivar::<id>("_popup_audio_scheme");
				let set_popup_audio_scheme_fn: extern fn(&mut Object, Sel, id) = Self::set_popup_audio_scheme_;
				decl.add_method(sel!(setPopup_audio_scheme:), set_popup_audio_scheme_fn);

				let get_popup_audio_scheme_fn: extern fn(&Object, Sel)->id = Self::get_popup_audio_scheme_;
				decl.add_method(sel!(popup_audio_scheme), get_popup_audio_scheme_fn);

				//property slide_volume
				decl.add_ivar::<id>("_slide_volume");
				let set_slide_volume_fn: extern fn(&mut Object, Sel, id) = Self::set_slide_volume_;
				decl.add_method(sel!(setSlide_volume:), set_slide_volume_fn);

				let get_slide_volume_fn: extern fn(&Object, Sel)->id = Self::get_slide_volume_;
				decl.add_method(sel!(slide_volume), get_slide_volume_fn);

				//property slide_pitch
				decl.add_ivar::<id>("_slide_pitch");
				let set_slide_pitch_fn: extern fn(&mut Object, Sel, id) = Self::set_slide_pitch_;
				decl.add_method(sel!(setSlide_pitch:), set_slide_pitch_fn);

				let get_slide_pitch_fn: extern fn(&Object, Sel)->id = Self::get_slide_pitch_;
				decl.add_method(sel!(slide_pitch), get_slide_pitch_fn);

				//property label_version
				decl.add_ivar::<id>("_label_version");
				let set_label_version_fn: extern fn(&mut Object, Sel, id) = Self::set_label_version_;
				decl.add_method(sel!(setLabel_version:), set_label_version_fn);				

				let get_label_version_fn: extern fn(&Object, Sel)->id = Self::get_label_version_;
				decl.add_method(sel!(label_version), get_label_version_fn);

				//methods
				let quit_fn: extern fn(&mut Object, Sel, id) = Self::quit_;
				decl.add_method(sel!(quit:), quit_fn);

				let value_changed_fn: extern fn(&mut Object, Sel, id) = Self::value_changed_;
				decl.add_method(sel!(value_changed:), value_changed_fn);

				let follow_link_fn: extern fn(&mut Object, Sel, id) = Self::follow_link_;
				decl.add_method(sel!(follow_link:), follow_link_fn);

				let windowWillClose_fn: extern fn(&Object, Sel, id) = Self::windowWillClose;
				decl.add_method(sel!(windowWillClose:), windowWillClose_fn);

				//let windowDidBecomeKey_fn: extern fn(&mut Object,Sel,id) = Self::windowDidBecomeKey;
				//decl.add_method(sel!(windowDidBecomeKey:), windowDidBecomeKey_fn);
			}

			decl.register();
		});


	    let cls = Class::get("SettingsDelegate").unwrap();
	    unsafe 
	    {
	       	let obj: id = msg_send![cls, new];	       
	       	obj.retain();
	       	let _:id = msg_send![obj, setUser_data: ptr_to_app];

	       	let data: *mut App = msg_send![obj, user_data];
	       	assert!(data == ptr_to_app);

			let nib_name = NSString::alloc(nil).init_str("Settings");
			let _: id = msg_send![class("NSBundle"), loadNibNamed:nib_name owner: obj];	

			Self::load_values(obj);

	       obj
    	}    
	}

	//property ptr_to_app
	extern fn set_user_data_(this: &mut Object, _cmd: Sel, val: usize){unsafe { this.set_ivar::<usize>("_user_data", val); }}
	extern fn get_user_data_(this: &Object, _cmd: Sel) -> usize{unsafe { *this.get_ivar::<usize>("_user_data") }}

	//property popup_audio_scheme
	extern fn set_popup_audio_scheme_(this: &mut Object, _cmd: Sel, val: id){unsafe { this.set_ivar::<id>("_popup_audio_scheme", val); }}
	extern fn get_popup_audio_scheme_(this: &Object, _cmd: Sel) -> id{unsafe { *this.get_ivar::<id>("_popup_audio_scheme") }}

	//property slide_volume
	extern fn set_slide_volume_(this: &mut Object, _cmd:Sel, val: id){unsafe{this.set_ivar::<id>("_slide_volume", val);}}
	extern fn get_slide_volume_(this: &Object, _cmd:Sel) -> id{unsafe{*this.get_ivar::<id>("_slide_volume")}}

	//property slide_pitch
	extern fn set_slide_pitch_(this: &mut Object, _cmd:Sel, val: id){unsafe{this.set_ivar::<id>("_slide_pitch", val);}}
	extern fn get_slide_pitch_(this: &Object, _cmd:Sel) -> id{unsafe{*this.get_ivar::<id>("_slide_pitch")}}

	extern fn set_label_version_(this: &mut Object, _cmd: Sel, val: id){unsafe{this.set_ivar::<id>("_label_version", val);}}
	extern fn get_label_version_(this: &Object, _cmd: Sel)->id{unsafe{*this.get_ivar::<id>("_label_version")}}
	

	extern fn quit_(this: &mut Object, _cmd: Sel, sender: id)
	{
		println!("Quit");
		unsafe
		{
			let _:id = msg_send![NSApp(), terminate: nil];
		}
	}

	extern fn follow_link_(this: &mut Object, _cmd: Sel, sender: id)
	{
		unsafe
		{
			let tag:i64 = msg_send![sender, tag];
			let url = match tag
			{
				0 => "http://www.yingDev.com/projects/Tickeys",
				1 => "http://www.yingdev.com/home/donate",
				_ => panic!("SettingsDelegate::follow_link_")
			};

			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			let url:id = msg_send![class("NSURL"), 
			URLWithString: NSString::alloc(nil).init_str(url)];

			msg_send![workspace, openURL: url]
		}
	}
	extern fn value_changed_(this: &mut Object, _cmd:Sel, sender: id)
	{
		println!("SettingsDelegate::value_changed_");

		const TAG_POPUP_SCHEME: i64 = 0;
		const TAG_SLIDE_VOLUME: i64 = 1; 
		const TAG_SLIDE_PITCH: i64 = 2;

		unsafe
		{
			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			println!("0");
			let app_ptr:&mut App = msg_send![this, user_data];
			println!("1");
			let tag:i64 = msg_send![sender, tag];
			match tag
			{
				TAG_POPUP_SCHEME => 
				{
					let value:i32 = msg_send![sender, indexOfSelectedItem];
					let scheme;
					
					//todo: dirty
					match value
					{
						0 => 
						{
							scheme = "bubble";
							app_ptr.load_audio(&(get_data_path("bubble")), &BUBBLE_AUDIO_FILES);
							let mut keymap:HashMap<u8, u8> = HashMap::new();
							keymap.insert(36, 8);
							app_ptr.set_keymap(keymap, BUBBLE_NON_UNIQ_AUDIO_COUNT);
						},
						1 => 
						{
							scheme = "typewriter";
							app_ptr.load_audio(&(get_data_path("typewriter")), &TYPEWRITER_AUDIO_FILES);
							let mut keymap:HashMap<u8, u8> = HashMap::new();
							keymap.insert(36, 9);
							keymap.insert(49, 5);
							keymap.insert(51, 8);
							app_ptr.set_keymap(keymap, TYPEWRITER_NON_UNIQ_AUDIO_COUNT);
						},
						2 => 
						{
							scheme = "mechanical";
							app_ptr.load_audio(&(get_data_path("mechanical")), &MECHANICAL_AUDIO_FILES);
							let mut keymap:HashMap<u8, u8> = HashMap::new();
							keymap.insert(36, 4);
							app_ptr.set_keymap(keymap, MECHANICAL_NON_UNIQ_AUDIO_COUNT);
						},
						_ => panic!("value_changed_")
					}

					let _:id = msg_send![user_defaults, setObject: NSString::alloc(nil).init_str(scheme.to_string().as_ref()) 
														   forKey: NSString::alloc(nil).init_str("audio_scheme")];
				},

				TAG_SLIDE_VOLUME =>
				{
					let value:f32 = msg_send![sender, floatValue];
					app_ptr.set_volume(value);

					let _:id = msg_send![user_defaults, setFloat: value forKey: NSString::alloc(nil).init_str("volume")];
				},

				TAG_SLIDE_PITCH =>
				{
					let value:f32 = msg_send![sender, floatValue];
					app_ptr.set_pitch(value);

					let _:id = msg_send![user_defaults, setFloat: value forKey: NSString::alloc(nil).init_str("pitch")];
				}

				_ => {panic!("WTF");}
			}
		}
		
	}

	extern fn windowWillClose(this: &Object, _cmd: Sel, note: id)
	{
		println!("SettingsDelegate::windowWillClose");
		unsafe
		{
			println!("-1");
			let app_ptr: *mut App = msg_send![this, user_data];
			println!("-2");
			(*app_ptr).showing_gui = false;
			println!("0");

			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			println!("1");
			let _:id = msg_send![user_defaults, synchronize];
println!("2");
			let _:id = msg_send![this, release];
			println!("3");
		}
	}

	unsafe fn load_values(this: id)
	{
		println!("loadValues");
		let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];

		let popup_audio_scheme: id = msg_send![this, popup_audio_scheme];
		let _: id = msg_send![popup_audio_scheme, removeAllItems];
		let _: id = msg_send![popup_audio_scheme, addItemWithTitle: NSString::alloc(nil).init_str("冒泡")];
		let _: id = msg_send![popup_audio_scheme, addItemWithTitle: NSString::alloc(nil).init_str("打字机")];
		let _: id = msg_send![popup_audio_scheme, addItemWithTitle: NSString::alloc(nil).init_str("机械键盘")];

		let pref = Pref::load();
		match pref.audio_scheme.as_ref()
		{
			"bubble" => {let _:id = msg_send![popup_audio_scheme, selectItemAtIndex:0];},
			"typewriter" => {let _:id = msg_send![popup_audio_scheme, selectItemAtIndex:1];},
			"mechanical" => {let _:id = msg_send![popup_audio_scheme, selectItemAtIndex: 2];},
			_ => {panic!("load_values: scheme={}", pref.audio_scheme);} 
		}

		let slide_volume: id = msg_send![this, slide_volume];
		let _:id = msg_send![slide_volume, setFloatValue: pref.volume];

		let slide_pitch: id = msg_send![this, slide_pitch];
		let _:id = msg_send![slide_pitch, setFloatValue: pref.pitch];

		let label_version: id = msg_send![this, label_version];
		let _:id = msg_send![label_version, setStringValue:NSString::alloc(nil).init_str(format!("v{}",CURRENT_VERSION).as_ref())];
	}

}

impl SettingsDelegate for id
{
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





