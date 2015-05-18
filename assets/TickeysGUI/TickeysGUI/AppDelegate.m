//
//  AppDelegate.m
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import "AppDelegate.h"
#import "SettingsDelegate.h"

@interface AppDelegate ()

@property (retain) IBOutlet NSWindow *window;
@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
	// Insert code here to initialize your application
	
	SettingsDelegate* del = [[SettingsDelegate alloc] init];
	[del retain];
	
	[NSBundle loadNibNamed:@"Settings" owner:del];
}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
	// Insert code here to tear down your application
}

@end
