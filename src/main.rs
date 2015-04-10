#![feature(collections)]

extern crate libc;
extern crate openal;
extern crate cocoa;
extern crate time;

#[macro_use]
extern crate objc;

use libc::{c_void};
use std::collections::VecMap;//A map optimized for small integer keys.
use std::option::Option;

mod core_graphics;
mod core_foundation;
mod alut;

use core_foundation::*;
use core_graphics::*;
use openal::al::*;
use openal::al::ffi::*;
use alut::*;
use cocoa::appkit::{};
use cocoa::foundation::{NSString};
use objc::*;
use cocoa::base::{class,id,nil};


static mut eventTap: CFMachPortRef = 0 as CFMachPortRef;  
static mut runLoopSource: CFRunLoopSourceRef = 0 as CFRunLoopSourceRef; 

/*static AUDIO_FILES: [&'static str; 26] = ["a.wav","b.wav","c.wav","d.wav","e.wav","f.wav","g.wav","h.wav",
			"i.wav","j.wav","k.wav","l.wav","m.wav","n.wav","o.wav","p.wav","q.wav","r.wav","s.wav",
			"t.wav","u.wav","v.wav","w.wav","x.wav","y.wav","z.wav"];
const NON_UNIQ_AUDIO_COUNT:u16 = 26;*/
	
static AUDIO_FILES: [&'static str; 9] = ["1.wav","2.wav","3.wav","4.wav","5.wav","6.wav","7.wav","8.wav", "enter.wav"];
const NON_UNIQ_AUDIO_COUNT:u16 = 9;


/*static AUDIO_FILES: [&'static str; 10] =["key-new-01.wav", "key-new-02.wav", "key-new-03.wav","key-new-04.wav", "key-new-05.wav",
	 "return-new.wav","scrollDown.wav", "scrollUp.wav"  , "space-new.wav" ,"backspace.wav"];
const NON_UNIQ_AUDIO_COUNT:u16 = 10;*/

static mut ptr_to_audios: *const Vec<AudioData> = 0 as *const _;

fn main() 
{	
	//todo: better option?
	let audios = &load_audio(&find_data_path("bubble"), &AUDIO_FILES);
	unsafe{ ptr_to_audios = std::mem::transmute(audios); }

	install_keyboard_tap();

	show_notification();

	run();
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

fn ensure_singleton()
{

}

fn show_notification()
{
	unsafe
	{
		//todo: leak
		let note:id = msg_send![NSUserNotification::new(nil), autorelease];
		note.setTitle_(NSString::alloc(nil).init_str("Tickeys正在运行"));
		note.setInformativeText_(NSString::alloc(nil).init_str("随时按 QAZ123 退出"));

		let center:id = msg_send![class("NSUserNotificationCenter"), defaultUserNotificationCenter];
		msg_send![center, deliverNotification: note]
	}
	

}

fn install_keyboard_tap()
{
	unsafe 
	{
        eventTap = CGEventTapCreate(CGEventTapLocation::kCGHIDEventTap, 
					CGEventTapPlacement::kCGHeadInsertEventTap, 
					CGEventTapOptions::kCGEventTapOptionListenOnly,
					CGEventMaskBit!(CGEventType::kCGEventKeyDown),
					handle_keyboard_event,
					std::ptr::null_mut());

        println!("eventTap:{:p}", eventTap);
        if eventTap == (0 as CFMachPortRef)
        {
        	panic!("failed to CGEventTapCreate");
        }

        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0 );
        CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource,  kCFRunLoopCommonModes);
	}
}

fn run()
{
	unsafe
	{
		CFRunLoopRun();
	}
}

fn load_audio(dir: &str, files: &[&str]) -> Vec<AudioData>
{
	unsafe
	{
		alutInit(std::ptr::null_mut(), std::ptr::null_mut());
	}

	let mut ret: Vec<AudioData> = Vec::new();
	let mut path = dir.to_string();
	path.push_str("/");
	let base_path_len = path.chars().count();

	for f in files.iter()
	{
		path.push_str(f);
		println!("loading audio:{}", path);
		let audio = AudioData::from_file(&path);
		path.truncate(base_path_len);

		ret.push(audio);
	}

	ret
}

extern fn handle_keyboard_event(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef
{
	let keycode = unsafe{CGEventGetIntegerValueField(event, CGEventField::kCGKeyboardEventKeycode)} as u16;

	play_keycode_audio(keycode);

	event
}

fn play_keycode_audio(keycode: u16)
{		
	static mut last_time: u64 = 0;
	
	let audios:&'static mut Vec<AudioData> = unsafe{std::mem::transmute(ptr_to_audios)};

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

	audios[index].play();
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












