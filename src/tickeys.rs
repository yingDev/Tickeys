extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate hyper;
extern crate toml;
extern crate block;
extern crate rustc_serialize;
//#[macro_use]
extern crate objc;
extern crate std;

use std::collections::{VecDeque, HashMap};
use std::option::Option;
use std::any::Any;
use std::boxed::Box;
use std::thread;
use std::io::Read;
use std::sync::{Once, ONCE_INIT};
use std::string::String;
use std::fs::File;
use std::ptr;

use libc::{c_void};
use core_foundation::*;
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use objc::*;
use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::{NSUInteger, NSRect, NSPoint, NSSize,NSAutoreleasePool, NSProcessInfo, NSString};
use cocoa::appkit::{NSApp,NSApplication, NSApplicationActivationPolicyRegular,NSWindow, NSTitledWindowMask, NSBackingStoreBuffered,NSMenu, NSMenuItem};

use hyper::Client;
use hyper::header::{Connection, ConnectionOption};
use hyper::status::StatusCode;

use self::block::{Block, ConcreteBlock};
use rustc_serialize::json;

use event_tap;

//自己的modules才需要声明
//mod core_graphics;
//mod core_foundation;
//mod alut;
//mod event_tap;

#[derive(RustcDecodable, RustcEncodable)]
pub struct AudioScheme
{
	pub name:String,
	pub display_name: String,
	pub files: Vec<String>,
	pub non_unique_count: u8,
	pub key_audio_map: HashMap<u8, u8>
}

pub struct Tickeys
{
	volume:f32,
	pitch:f32,

	audio_data: Vec<AudioData>,
	keymap: HashMap<u8, u8>,
	first_n_non_unique: i16,

	last_keys: VecDeque<u8>,
	//keyseq_registry: HashMap<u8, VecDeque<u8>>,
	
	keyboard_monitor: Option< event_tap::KeyboardMonitor>, //defered

	on_keydown: Option<fn(sender:&Tickeys, key: u8)>,
	//pub on_keyseq: Option<fn(sender:&Tickeys, seq_id:u8)>
}

impl Tickeys
{
	pub fn new() -> Tickeys
	{
		unsafe
		{
			alutInit(std::ptr::null_mut(), std::ptr::null_mut());
		}

		unsafe
		{

			let mut tk = Tickeys{
				volume:1f32,
				pitch:1f32, 
				audio_data: Vec::new(), 
				keymap: HashMap::new(),
				first_n_non_unique: -1,
				last_keys: VecDeque::new(), 
				//keyseq_registry: HashMap::new(),
				keyboard_monitor:None, 
				on_keydown: Option::None,
				//on_keyseq: Option::None
			};
			tk
		}
	}

	pub fn start(&mut self)
	{
		let mut tap;

		let ptr_to_self: *mut c_void = unsafe{std::mem::transmute(self)};

		unsafe
		{
			let mut tap_result = event_tap::KeyboardMonitor::new(Tickeys::handle_keyboard_event, ptr_to_self);
			match tap_result
			{
				Ok(t) => tap = t,
				Err(msg) => panic!("error: KeyboardMonitor::new: {}", msg)
			}

			let self_:&mut Tickeys = std::mem::transmute(ptr_to_self);
			self_.keyboard_monitor = Some(tap);
		}
	}

	pub fn stop(&mut self)
	{
		//todo: stop the kbd monitor?
	}

	pub fn load_scheme(&mut self, dir: &str, scheme: &AudioScheme)
	{
		self.audio_data.clear();

		let mut path = dir.to_string();
		path.push_str("/");
		let base_path_len = path.chars().count();

		for f in scheme.files.iter()
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

		self.keymap = scheme.key_audio_map.clone();
		self.first_n_non_unique = scheme.non_unique_count as i16;
	}

	pub fn set_volume(&mut self, volume: f32)
	{
		if volume == self.volume {return;}
		//todo:
		self.volume = volume;
		for audio in self.audio_data.iter_mut()
		{
			audio.set_gain(volume);
		}
	}

	pub fn set_pitch(&mut self, pitch: f32)
	{
		if pitch == self.pitch {return;}
		//todo:
		self.pitch = pitch;
		for audio in self.audio_data.iter_mut()
		{
			audio.set_pitch(pitch);
		}
	}

	pub fn get_volume(&self) -> f32
	{
		self.volume
	}

	pub fn get_pitch(&self) -> f32
	{
		self.pitch
	}

	pub fn get_last_keys(&self) -> &VecDeque<u8>
	{
		&self.last_keys
	}

	extern fn handle_keyboard_event(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef
	{
		let keycode = unsafe{CGEventGetIntegerValueField(event, CGEventField::kCGKeyboardEventKeycode)} as u16;

		assert!(refcon != 0 as *mut c_void);

		//todo: temp
		let tickeys: &mut Tickeys = unsafe{ std::mem::transmute(refcon)};
		tickeys.handle_keydown(keycode as u8);
		//app.audios[(keycode % 8) as usize].play();
		//play_keycode_audio(keycode);

		event
	}

	fn handle_keydown(&mut self, keycode: u8)
	{	
		self.last_keys.push_back(keycode);
		if self.last_keys.len() > 6  //todo: make the length configurable
		{
			self.last_keys.pop_front();
		}

		/*if self.last_keys.iter().zip(QUIT_KEY_SEQ.iter()).filter(|&(a,b)| a == b).count() == QUIT_KEY_SEQ.len()
		{
			self.show_settings();
		}*/
		match self.on_keydown
		{
			None => {},
			Some(f) => f(self, keycode)
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

	pub fn set_on_keydown(&mut self, on_keydown: Option<fn(sender:&Tickeys, key: u8)>)
	{
		self.on_keydown = on_keydown;
	}

	/*fn register_keyseq(&mut self, seq_id:u8, seq:VecDeque<u8>)
	{
		self.keyseq_registry.insert(seq_id,seq);
	}

	fn remove_keyseq(&mut self, seq_id:u8)
	{
		self.keyseq_registry.remove(&seq_id);
	}*/

	/*fn set_keymap(&mut self, keymap: HashMap<u8, u8>, first_n_non_unique: u8)
	{
		self.keymap = keymap;
		self.first_n_non_unique = first_n_non_unique as i16;
	}*/


	/*fn show_settings(&mut self)
	{
		println!("Settings!");

		if self.showing_gui
		{
			return;
		}
		self.showing_gui = true;
					
		let settings_delegate = SettingsDelegate::new(nil, self);

	}*/

}

pub struct AudioData
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

	pub fn play(&mut self)
	{
		unsafe
		{
			alSourcePlay(self.source);
		}
	}

	pub fn set_pitch(&mut self, value: f32)
	{
    	unsafe{alSourcef(self.source, AL_PITCH, value);}
	}

	pub fn set_gain(&mut self, value: f32)
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
