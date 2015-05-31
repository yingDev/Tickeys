#[allow(non_snake_case)]
#[allow(non_camel_case_types)]

extern crate openal;
use openal::al::*;

//#[link(name= "alut")]
#[allow(dead_code)]
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