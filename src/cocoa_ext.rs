extern crate objc;
use cocoa::base::{class,id,nil};


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