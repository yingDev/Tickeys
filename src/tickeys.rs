extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;
extern crate rustc_serialize;
extern crate objc;
extern crate std;

use std::collections::{VecDeque, HashMap, BTreeMap};
use std::option::Option;
use std::io::Read;
use std::string::String;

use libc::{c_void};
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use objc::*;

use event_tap;

#[derive(RustcDecodable, RustcEncodable)]
pub struct AudioScheme
{
	pub name:String,
	pub display_name: String,
	pub files: Vec<String>,
	pub non_unique_count: u8,
	pub key_audio_map: BTreeMap<u8, u8>
}

pub struct Tickeys
{
	volume:f32,
	pitch:f32,

	audio_player: SimpleAudioPlayer,
	keymap: BTreeMap<u8, u8>,
	first_n_non_unique: i16,

	last_keys: VecDeque<u8>,
	
	keyboard_monitor: Option< event_tap::KeyboardMonitor>, //defered

	on_keydown: Option<fn(sender:&Tickeys, key: u8)>,
}

impl Tickeys
{
	pub fn new() -> Tickeys
	{
		unsafe
		{
			alutInit(std::ptr::null_mut(), std::ptr::null_mut());
		}

		Tickeys{
			volume:1f32,
			pitch:1f32, 
			audio_player: SimpleAudioPlayer::new(4), 
			keymap: BTreeMap::new(),
			first_n_non_unique: -1,
			last_keys: VecDeque::with_capacity(8), 
			keyboard_monitor:None, 
			on_keydown: Option::None		
		}
	}

	pub fn start(&mut self)
	{
		let mut tap;

		let ptr_to_self: *mut c_void = unsafe{std::mem::transmute(self)};

		unsafe
		{
			let tap_result = event_tap::KeyboardMonitor::new(Tickeys::handle_keyboard_event, ptr_to_self);

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
		let mut audio_data = Vec::with_capacity(scheme.files.len());

		let mut path = dir.to_string() + "/";
		let base_path_len = path.chars().count();

		for f in scheme.files.iter()
		{
			path.push_str(f);
			println!("loading audio:{}", path);
			let mut audio = AudioData::from_file(&path);

			if audio.buffer == 0 as ALuint
			{
				panic!("failed to load audio file:{}", f);
			}

			path.truncate(base_path_len);
			
			audio_data.push(audio);
		}

		self.audio_player.load_data(audio_data);
		
		self.audio_player.set_gain(self.volume);
		self.audio_player.set_pitch(self.pitch);
		
		self.keymap = scheme.key_audio_map.clone();
		self.first_n_non_unique = scheme.non_unique_count as i16;
	}

	pub fn set_volume(&mut self, volume: f32)
	{
		if volume == self.volume {return;}
		self.volume = volume;
		
		self.audio_player.set_gain(volume);
	}

	pub fn set_pitch(&mut self, pitch: f32)
	{
		if pitch == self.pitch {return;}
		self.pitch = pitch;

		self.audio_player.set_pitch(pitch);
	}

	#[allow(dead_code)]
	pub fn get_volume(&self) -> f32
	{
		self.volume
	}

	#[allow(dead_code)]
	pub fn get_pitch(&self) -> f32
	{
		self.pitch
	}

	pub fn get_last_keys(&self) -> &VecDeque<u8>
	{
		&self.last_keys
	}

	#[allow(unused_variables)]
	extern fn handle_keyboard_event(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef
	{
		let keycode = unsafe{CGEventGetIntegerValueField(event, CGEventField::kCGKeyboardEventKeycode)} as u16;

		assert!(refcon != 0 as *mut c_void);

		let tickeys: &mut Tickeys = unsafe{ std::mem::transmute(refcon)};
		tickeys.handle_keydown(keycode as u8);

		event
	}

	fn handle_keydown(&mut self, keycode: u8)
	{	
		self.last_keys.push_back(keycode);
		if self.last_keys.len() > 6  //todo: make the length configurable
		{
			self.last_keys.pop_front();
		}

		match self.on_keydown
		{
			None => {},
			Some(f) => f(self, keycode)
		}

		//println!("key:{}", keycode);

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
		
		if self.is_too_frequent(keycode)
		{
			return;
		}

		if index == -1 
		{
			return;
		}

		self.audio_player.play(index as usize);
	}

	pub fn set_on_keydown(&mut self, on_keydown: Option<fn(sender:&Tickeys, key: u8)>)
	{
		self.on_keydown = on_keydown;
	}

	fn is_too_frequent(&self, keycode: u8) -> bool
	{
		unsafe
		{
			static mut last_time: u64 = 0;
			static mut last_key: i16 = -1;
			let now = time::precise_time_ns() / 1000 / 1000;

			let delta = now - last_time ;

			if delta < 120 && last_key == (keycode as i16)
			{
				last_time = now;
				return true;
			}
			last_key = keycode as i16;
			last_time = now;

			return false;
		}

	}

}

pub struct AudioData
{
	buffer: ALuint,
}

impl AudioData
{
	//todo: how to handle error?
	pub fn from_file(file: &str) -> AudioData
	{
		let file_ptr = std::ffi::CString::new(file).unwrap().as_ptr();
		let mut audio = AudioData{buffer:0};
		
		unsafe
		{
			audio.buffer = alutCreateBufferFromFile(file_ptr);
			// Create sound source (use buffer to fill source)
    		//alGenSources(1, &mut audio.source);
    		//alSourcei(audio.source, AL_BUFFER, audio.buffer as ALint);

    		if audio.buffer == 0
    		{
    			panic!("failed to load file [{}]: {}", alutGetError() ,file);
    		}
		}

		audio
	}

	pub fn id(&self) -> ALuint
	{
		self.buffer
	}

}

impl Drop for AudioData
{
	fn drop(&mut self)
	{
		unsafe
		{
    		alDeleteBuffers(1, &self.buffer);
		}
		
	}
}

struct AudioSource 
{
	id: ALuint,
}

impl AudioSource 
{
	pub fn new() -> Option<AudioSource>
	{
		let mut id = 0;
		unsafe{ alGenSources(1, &mut id); }

		match unsafe { alGetError() }
		{
			AL_NO_ERROR => Some(AudioSource{id: id}),
			_ => None
		}
	}

	pub fn connect_to_buffer(&mut self, data: &AudioData)
	{
		self.stop();
		unsafe
		{ 
			alSourcei(self.id, AL_BUFFER, data.id() as ALint); 

		}
	}

	pub fn disconnect_from_buffer(&mut self)
	{
		unsafe
		{		
			alSourceStop(self.id);
			alSourcei(self.id, AL_BUFFER, 0);
		}
	}

	pub fn set_gain(&mut self, gain: f32)
	{
		unsafe{ alSourcef(self.id, AL_GAIN, gain); }
	}

	pub fn set_pitch(&mut self, pitch: f32)
	{
		unsafe{alSourcef(self.id, AL_PITCH, pitch);}
	}

	pub fn play(&mut self)
	{
		unsafe{ alSourcePlay(self.id); }
	}

	pub fn stop(&mut self)
	{
		unsafe{ alSourceStop(self.id); }
	}

	//pub fn state()
}

impl Drop for AudioSource
{
	fn drop(&mut self)
	{
		self.stop();
		unsafe{ alDeleteSources(1, &self.id); }
	}
}

struct SimpleAudioPlayer
{
	data: Vec<AudioData>,
	source_cache: VecDeque<AudioSource>,
	max_source_count: usize,
}

impl SimpleAudioPlayer
{
	pub fn new(max_source_count: usize) -> SimpleAudioPlayer
	{
		assert!(max_source_count > 0);

		let mut sources = VecDeque::with_capacity(max_source_count);
		for _ in 0..max_source_count
		{
			sources.push_back(AudioSource::new().unwrap());
		}

		SimpleAudioPlayer{data: Vec::new(), source_cache: sources, max_source_count: max_source_count}
	}

	pub fn load_data(&mut self, data: Vec<AudioData>)
	{
		for s in self.source_cache.iter_mut()
		{
			s.disconnect_from_buffer();
		}

		self.data = data;
	}

	pub fn set_gain(&mut self, gain: f32)
	{
		for s in self.source_cache.iter_mut()
		{
			s.set_gain(gain);
		}
	}

	pub fn set_pitch(&mut self, pitch: f32)
	{
		for s in self.source_cache.iter_mut()
		{
			s.set_pitch(pitch);
		}
	}

	pub fn play(&mut self, index: usize)
	{
		let data = match self.data.get(index)
		{
			Some(val) => val,
			None => return
		};

		let mut oldest_source = self.source_cache.pop_front().unwrap();

		oldest_source.connect_to_buffer(&data);
		oldest_source.play();

		self.source_cache.push_back(oldest_source);

	}

	pub fn unload_data(&mut self)
	{
		self.source_cache.clear();
		self.data.clear();
	}
}

impl Drop for SimpleAudioPlayer
{
	fn drop(&mut self)
	{
		self.unload_data();
	}
}









