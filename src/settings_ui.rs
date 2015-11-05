extern crate objc;

use super::consts::*;
use cocoa_ext::*;
use pref::*;
use tickeys::Tickeys;
use std::sync::{ONCE_INIT, Once};

use objc::runtime::*;
use cocoa::base::{class,id,nil};
use cocoa::foundation::NSString;
use cocoa::appkit::NSApp;

// naive way of make this a singleton
static mut SHOWING_GUI:bool = false;


#[allow(non_snake_case)]
#[allow(unused_variables)]
pub trait SettingsDelegate
{
	fn get_instance(_: Self, ptr_to_app: *mut Tickeys) -> id
	{
		Self::__register_objc_class_once();


	    unsafe
	    {
	    	if SHOWING_GUI 
			{
				return nil;
			}
	    	
	    	let cls = Class::get("SettingsDelegate").unwrap();

	       	let obj: id = msg_send![cls, new];
	       	obj.retain();
	       	let _:id = msg_send![obj, setUser_data: ptr_to_app];

	       	let data: *mut Tickeys = msg_send![obj, user_data];
	       	assert!(data == ptr_to_app);

			let nib_name = NSString::alloc(nil).init_str("Settings");
			let _: id = msg_send![class("NSBundle"), loadNibNamed:nib_name owner: obj];

			Self::load_values(obj);

			SHOWING_GUI = true;
	       	obj
    	}
	}

	fn __register_objc_class_once()
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

				//property window
				decl.add_ivar::<id>("_window");
				let set_window_fn: extern fn(&mut Object, Sel, id) = Self::set_window_;
				decl.add_method(sel!(setWindow:), set_window_fn);

				let get_window_fn: extern fn(& Object, Sel)->id = Self::get_window_;
				decl.add_method(sel!(getWindow), get_window_fn);

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

	//property label_version
	extern fn set_label_version_(this: &mut Object, _cmd: Sel, val: id){unsafe{this.set_ivar::<id>("_label_version", val);}}
	extern fn get_label_version_(this: &Object, _cmd: Sel)->id{unsafe{*this.get_ivar::<id>("_label_version")}}

	//property window
	extern fn set_window_(this: &mut Object, _cmd: Sel, val: id){unsafe{this.set_ivar::<id>("_window", val);}}
	extern fn get_window_(this: &Object, _cmd: Sel)->id{unsafe{*this.get_ivar::<id>("_window")}}

	extern fn quit_(this: &mut Object, _cmd: Sel, sender: id)
	{
		println!("Quit");
		app_terminate();
	}

	extern fn follow_link_(this: &mut Object, _cmd: Sel, sender: id)
	{
		unsafe
		{
			let tag:i64 = msg_send![sender, tag];
			let url = match tag
			{
				0 => WEBSITE,
				1 => DONATE_URL,
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
			let tickeys_ptr:&mut Tickeys = msg_send![this, user_data];
			let tag:i64 = msg_send![sender, tag];

			match tag
			{
				TAG_POPUP_SCHEME =>
				{

					let value:i32 = msg_send![sender, indexOfSelectedItem];
					
					let sch;// = &schemes[value as usize];
					{
						let schemes = tickeys_ptr.get_schemes();//= load_audio_schemes();
						sch = schemes[value as usize].name.clone();
					}
					

					let scheme_dir = "data/".to_string() + &sch;//.to_string();
					//scheme_dir.push_str(&sch.name);
					tickeys_ptr.load_scheme(&get_res_path(&scheme_dir), &sch);

					let _:id = msg_send![user_defaults, setObject: NSString::alloc(nil).init_str(sch.as_ref())
														   forKey: NSString::alloc(nil).init_str("audio_scheme")];
				},

				TAG_SLIDE_VOLUME =>
				{
					let value:f32 = msg_send![sender, floatValue];
					tickeys_ptr.set_volume(value);

					let _:id = msg_send![user_defaults, setFloat: value forKey: NSString::alloc(nil).init_str("volume")];
				},

				TAG_SLIDE_PITCH =>
				{
					let mut value:f32 = msg_send![sender, floatValue];
					if value > 1f32
					{
						//just map [1, 1.5] -> [1, 2]
						value = value * (2.0f32/1.5f32);
					}
					tickeys_ptr.set_pitch(value);

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
			let app_ptr: *mut Tickeys = msg_send![this, user_data];
			SHOWING_GUI = false;

			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			let _:id = msg_send![user_defaults, synchronize];
			let _:id = msg_send![this, release];
		}
	}

	unsafe fn load_values(this: id)
	{
		println!("loadValues");
		let app_ptr: &mut Tickeys = msg_send![this, user_data];
		let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
		let popup_audio_scheme: id = msg_send![this, popup_audio_scheme];
		let _: id = msg_send![popup_audio_scheme, removeAllItems];

		let schemes = app_ptr.get_schemes();//load_audio_schemes();
		let pref = Pref::load(schemes);

		for i in 0..schemes.len()
		{
			let s = &schemes[i];

			let _: id = msg_send![popup_audio_scheme, addItemWithTitle: ns_localized_string(&s.display_name)];
			if  *s.name == pref.audio_scheme
			{
				let _:id = msg_send![popup_audio_scheme, selectItemAtIndex:i];
			}
		}

		let slide_volume: id = msg_send![this, slide_volume];
		let _:id = msg_send![slide_volume, setFloatValue: pref.volume];

		let slide_pitch: id = msg_send![this, slide_pitch];
		let value =  if pref.pitch > 1f32
		{
			pref.pitch * (1.5f32/2.0f32)
		} else
		{
			pref.pitch
		};
		let _:id = msg_send![slide_pitch, setFloatValue: value];

		let label_version: id = msg_send![this, label_version];
		let _:id = msg_send![label_version, setStringValue:NSString::alloc(nil).init_str(format!("v{}",CURRENT_VERSION).as_ref())];

		//let _:id = msg_send![this, show]

		println!("makeKeyAndOrderFront:");
		let win:id = msg_send![this, getWindow];
		let _:id = msg_send![win, makeKeyAndOrderFront:nil];
		let _:id = msg_send![NSApp(), activateIgnoringOtherApps:true];
	}

}

impl SettingsDelegate for id
{
}