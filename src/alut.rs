#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]

extern crate openal;
use openal::al::*;

//#[link(name= "alut")]
extern "cdecl"
{
	pub fn alutInit(argcp:*mut i32, argv: *mut *mut u8) -> ALboolean;
 	pub fn alutCreateBufferFromFile(fileName:*const i8) -> ALuint;
 	pub fn alutGetError() -> ALenum;
 	pub fn alutExit() -> ALboolean;
}

//todo: temp
pub const AL_BUFFER:ALenum = 0x1009;
pub const AL_PITCH:ALenum = 0x1003;
pub const AL_GAIN:ALenum = 0x100A;

pub const AL_SOURCE_STATE: ALenum = 0x1010;
pub const AL_INITIAL: ALenum = 0x1011;
pub const AL_PLAYING: ALenum = 0x1012;
pub const AL_PAUSED: ALenum = 0x1013;
pub const AL_STOPPED: ALenum = 0x1014;

pub const AL_NO_ERROR: ALenum = 0;