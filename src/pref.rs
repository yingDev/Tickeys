use cocoa::base::{class,id,nil};
use cocoa::foundation::NSString;
use tickeys::AudioScheme;
use cocoa_util::*;


pub struct Pref
{
	pub scheme: String,
	pub volume: f32,
	pub pitch: f32,
}

impl Pref
{
	pub fn load(schemes: &Vec<AudioScheme>) -> Pref
	{
		unsafe
		{
			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			let pref_exists_key:id = NSString::alloc(nil).init_str("pref_exists");

			let pref = Pref{scheme: schemes[0].name.clone(), volume: 0.5f32, pitch: 1.0f32};

			let pref_exists: id = msg_send![user_defaults, stringForKey: pref_exists_key];
			if pref_exists == nil //first run
			{
				pref.save();
				return pref;
			}else
			{
				let audio_scheme: id = msg_send![user_defaults,
					stringForKey: NSString::alloc(nil).init_str("audio_scheme")];

				let volume: f32 = msg_send![user_defaults,
					floatForKey: NSString::alloc(nil).init_str("volume")];

				let pitch: f32 = msg_send![user_defaults,
					floatForKey: NSString::alloc(nil).init_str("pitch")];

				let mut scheme_str = nsstring_to_string(audio_scheme);

				//validate scheme
				if schemes.iter().filter(|s|{*s.name == scheme_str}).count() == 0
				{
					scheme_str = pref.scheme;
				}

				Pref{scheme:  scheme_str, volume: volume, pitch: pitch}
			}
		}

	}

	pub fn save(&self)
	{
		unsafe
		{
			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];

			let _:id = msg_send![user_defaults,
				setObject: NSString::alloc(nil).init_str(&self.scheme)
				forKey: NSString::alloc(nil).init_str("audio_scheme")];

			let _:id = msg_send![user_defaults,
				setFloat: self.volume
				forKey: NSString::alloc(nil).init_str("volume")];

			let _:id = msg_send![user_defaults,
				setFloat: self.pitch forKey: NSString::alloc(nil).init_str("pitch")];

			let pref_exists_key:id = NSString::alloc(nil).init_str("pref_exists");
			let _:id = msg_send![user_defaults, setObject:pref_exists_key forKey: pref_exists_key];

			let _:id = msg_send![user_defaults, synchronize];
		}


	}
}
