#![feature(collections)]

extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;

#[macro_use]
extern crate objc;

use libc::{c_void};
use std::collections::{VecDeque, VecMap};
use std::option::Option;
use std::any::Any;
use std::boxed::Box;

mod core_graphics;
mod core_foundation;
mod alut;
mod event_tap;

use core_foundation::*;
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use cocoa::appkit::{};
use cocoa::foundation::{NSString};
use objc::*;
use cocoa::base::{class,id,nil};

const QUIT_KEY_SEQ: &'static[u8] = b"QAZ123";
	
static AUDIO_FILES: [&'static str; 9] = ["1.wav","2.wav","3.wav","4.wav","5.wav","6.wav","7.wav","8.wav", "enter.wav"];
const NON_UNIQ_AUDIO_COUNT:u16 = 8;

fn main() 
{	
	unsafe
	{
		alutInit(std::ptr::null_mut(), std::ptr::null_mut());
	}

	let mut app = App::new();
	app.load_audio(&find_data_path("bubble"), &AUDIO_FILES);

	//todo: not actually impled
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
	keyboard_monitor: Option< event_tap::KeyboardMonitor> //defered
}

impl App
{
	pub fn new() -> App
	{
		let mut app = App{audio_data: Vec::new(), special_keymap: VecMap::new(), last_keys: VecDeque::new(), keyboard_monitor:None};
		app
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


	fn show_notification(&self, title: &str, msg: &str)
	{
		unsafe
		{
			let note:id = msg_send![NSUserNotification::new(nil), autorelease];
			note.setTitle_(NSString::alloc(nil).init_str(title));
			note.setInformativeText_(NSString::alloc(nil).init_str(msg));

			let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
			msg_send![center, deliverNotification: note]
		}
	}

	extern fn handle_keyboard_event(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef
	{
		let keycode = unsafe{CGEventGetIntegerValueField(event, CGEventField::kCGKeyboardEventKeycode)} as u16;

		assert!(refcon != 0 as *mut c_void);

		//todo: temp
		let app: &mut App = unsafe{ std::mem::transmute(refcon)};
		app.play_keycode_audio(keycode);
		//app.audios[(keycode % 8) as usize].play();
		//play_keycode_audio(keycode);

		event
	}

	fn play_keycode_audio(&mut self, keycode: u16)
	{		
		static mut last_time: u64 = 0;
		
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
			if delta < 60
			{
				last_time = now;
				return;
			}
			last_time = now;
		}

		self.audio_data[index].play();
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












