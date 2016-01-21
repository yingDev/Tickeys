extern crate objc;

use super::consts::*;
use super::cocoa_util::*;
use pref::*;
use core_graphics::*;
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
pub trait SettingsController //<NSWindowDelegate, NSTableViewDelegate, NSTableViewDataSource>
{
	fn get_instance(_: Self, ptr_to_app: *mut Tickeys) -> id
	{
		Self::__register_objc_class_once();

	    unsafe
	    {
	    	if SHOWING_GUI { return nil };
			
			let nib_name = NSString::alloc(nil).init_str("Settings");
			let inst: id = msg_send![class(stringify!(SettingsController)), alloc];
	       	let inst: id = msg_send![inst, initWithWindowNibName: nib_name];
	       	inst.retain();

	       	let _:id = msg_send![inst, setUser_data: ptr_to_app];
			let _: id = msg_send![inst, showWindow: nil];


			SHOWING_GUI = true;
	       	inst
    	}
	}

	fn __register_objc_class_once()
	{
		static REGISTER_APPDELEGATE: Once = ONCE_INIT;
		REGISTER_APPDELEGATE.call_once(||
		{
			println!("SettingsController::__register_objc_class_once");
			let superCls = objc::runtime::Class::get("NSWindowController").unwrap();
			let mut decl = objc::declare::ClassDecl::new(superCls, stringify!(SettingsController)).unwrap();

			decl_prop!(decl, usize, user_data);
			decl_prop!(decl, id, popup_audio_scheme);
			decl_prop!(decl, id, slide_volume);
			decl_prop!(decl, id, slide_pitch);
			decl_prop!(decl, id, label_version);
			decl_prop!(decl, id, filterListTable);

			unsafe
			{
				//methods
				decl.add_method(sel!(quit:), Self::quit_ as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(value_changed:), Self::value_changed_ as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(follow_link:), Self::follow_link_ as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(windowWillClose:), Self::windowWillClose as extern fn(&Object, Sel, id));
				decl.add_method(sel!(windowDidLoad), Self::windowDidLoad as extern fn(&mut Object, Sel));
				decl.add_method(sel!(tableView:shouldEditTableColumn:row:), Self::tableViewShouldEditTableColumnRow as extern fn(&mut Object, Sel, id, id, i32) -> bool);

				decl.add_method(sel!(btnAddClicked:), Self::btnAddClicked as extern fn(&mut Object, Sel, id));
				decl.add_method(sel!(btnRemoveClicked:), Self::btnRemoveClicked as extern fn(&mut Object, Sel, id));


				//tableViewSource & tableViewDelegate
				decl.add_method(sel!(numberOfRowsInTableView:), Self::numberOfRowsInTableView as extern fn(&mut Object, Sel, id)->i32);
				decl.add_method(sel!(tableView:objectValueForTableColumn:row:), Self::tableViewObjectValueForTableColumn as extern fn(&mut Object, Sel, id, id, i32)->id);

			}

			decl.register();
		});
	}

	extern fn windowDidLoad(this: &mut Object, _cmd: Sel)
	{
		println!("windowDidLoad");
		unsafe
		{
			let window: id = msg_send![this, window];

			//hide window btns
			let btnMin: id = msg_send![window, standardWindowButton:1];
			let _: id = msg_send![btnMin, setHidden: true];
			let btnZoom: id = msg_send![window, standardWindowButton:2];
			let _: id = msg_send![btnZoom, setHidden: true];

			let _: id = msg_send![window, setLevel: CGWindowLevelForKey(CGWindowLevelKey::kCGFloatingWindowLevelKey)];

			Self::load_values(this);
		}

	}

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
				_ => panic!("SettingsController::follow_link_")
			};

			let workspace: id = msg_send![class("NSWorkspace"), sharedWorkspace];
			let url:id = msg_send![class("NSURL"),
			URLWithString: NSString::alloc(nil).init_str(url)];

			msg_send![workspace, openURL: url]
		}
	}

	extern fn value_changed_(this: &mut Object, _cmd:Sel, sender: id)
	{
		println!("SettingsController::value_changed_");

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

					let _:id = msg_send![user_defaults,
						setObject: NSString::alloc(nil).init_str(sch.as_ref())
						forKey: NSString::alloc(nil).init_str("audio_scheme")];
				},

				TAG_SLIDE_VOLUME =>
				{
					let value:f32 = msg_send![sender, floatValue];
					tickeys_ptr.set_volume(value);

					let _:id = msg_send![user_defaults, setFloat: value
						forKey: NSString::alloc(nil).init_str("volume")];
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

					let _:id = msg_send![user_defaults, setFloat: value
						forKey: NSString::alloc(nil).init_str("pitch")];
				}

				_ => {panic!("WTF");}
			}
		}

	}

	extern fn windowWillClose(this: &Object, _cmd: Sel, note: id)
	{
		println!("SettingsController::windowWillClose");
		unsafe
		{
			let app_ptr: *mut Tickeys = msg_send![this, user_data];
			SHOWING_GUI = false;

			let user_defaults: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
			let _:id = msg_send![user_defaults, synchronize];
			let _:id = msg_send![this, release];
		}
	}

	extern fn btnAddClicked(this: &mut Object, _cmd: Sel, sender: id)
	{
		println!("Add Item");
		unsafe
		{
			let open: id = msg_send![class("NSOpenPanel"), openPanel];
			let apps_dir: id = msg_send![class("NSURL"), URLWithString: nsstr("/Applications")];
			let allowed_types: id = msg_send![class("NSMutableArray"), arrayWithCapacity:2];
			let _: id = msg_send![allowed_types, addObject: nsstr("app")];

			let _: id = msg_send![open, setDirectoryURL: apps_dir];
			let _: id = msg_send![open, setAllowedFileTypes: allowed_types];
			let _: id = msg_send![open, setAllowsMultipleSelection: true];

			let ret: i32 = msg_send![open, runModal];
			if ret == 1 //Ok 
			{
				let files: id = msg_send![open, URLs];
				let n: i32 = msg_send![files, count];

				let filterList: id = Self::filterList();
				for i in 0..n 
				{
					let appName: id = nsurl_filename(msg_send![files, objectAtIndex: i]);

					let contains: bool = msg_send![filterList, containsObject: appName];
					if !contains
					{
						let _: id = msg_send![filterList, addObject: appName];
					}
				}

				//reload table ddata
				let filterListTable: id = msg_send![this, filterListTable];
				let _: id = msg_send![filterListTable, reloadData];

				//save 
				let ud: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
				let _: id = msg_send![ud, setObject:filterList forKey: nsstr("FilterList")];
			}
		}
	}

	extern fn btnRemoveClicked(this: &mut Object, _cmd: Sel, sender: id)
	{
		println!("Remove Item");
		unsafe
		{
			let filterList = Self::filterList();
			let table: id = msg_send![this, filterListTable];

			let selectedRow: i32 = msg_send![table, selectedRow];

			if selectedRow >= 0 
			{
				let _: id = msg_send![filterList, removeObjectAtIndex:selectedRow];
				let _: id = msg_send![table, reloadData];

				let ud: id = msg_send![class("NSUserDefaults"), standardUserDefaults];
				let _: id = msg_send![ud, setObject:filterList forKey: nsstr("FilterList")];
			}
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

			let _: id = msg_send![popup_audio_scheme, addItemWithTitle: l10n_str(&s.display_name)];
			if  *s.name == pref.scheme
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
		let _:id = msg_send![label_version,
			setStringValue: NSString::alloc(nil).init_str(format!("{}",CURRENT_VERSION).as_ref())];

		//let _:id = msg_send![this, show]

		println!("makeKeyAndOrderFront:");
		let win:id = msg_send![this, window];
		let _:id = msg_send![win, makeKeyAndOrderFront:nil];
		let _:id = msg_send![NSApp(), activateIgnoringOtherApps:true];
	}


	//-(NSInteger)numberOfRowsInTableView:(NSTableView *)tableView
	extern fn numberOfRowsInTableView(this: &mut Object, _cmd: Sel, tableView: id) -> i32
	{
		unsafe
		{
			msg_send![Self::filterList(), count]
		}
	}

	fn filterList() -> id
	{
		unsafe
		{
			//strong coupling...
			let appDelegate: id = msg_send![NSApp(), delegate];
			msg_send![appDelegate, filterList]
		}
	}

	//-(id)tableView:(NSTableView *)tableView objectValueForTableColumn:(NSTableColumn *)tableColumn row:(NSInteger)row
	extern fn tableViewObjectValueForTableColumn(this: &mut Object, _cmd: Sel, tableView: id, tableColumn: id, row: i32) -> id 
	{
		unsafe
		{
			msg_send![Self::filterList(), objectAtIndex: row]
		}
	}

	//-(BOOL)tableView:(NSTableView *)tableView shouldEditTableColumn:(NSTableColumn *)tableColumn row:(NSInteger)row
	extern fn tableViewShouldEditTableColumnRow(this: &mut Object, _cmd: Sel, tableView: id, tableColumn: id, row: i32) -> bool 
	{
		false
	}
}

impl SettingsController for id
{
}
