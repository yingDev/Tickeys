//
//  main.m
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import <Cocoa/Cocoa.h>
#import "AppDelegate.h"

int main(int argc, const char * argv[]) {
	
	//return NSApplicationMain(argc, argv);
	
	@autoreleasepool
	{
		
		NSApp = [NSApplication sharedApplication];
		AppDelegate* del = [[[AppDelegate alloc] init] autorelease];
		
		[NSApp setDelegate:del];
		
		[NSApp run];
	}
	
}
