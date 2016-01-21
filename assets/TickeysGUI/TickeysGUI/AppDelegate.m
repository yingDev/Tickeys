//
//  AppDelegate.m
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import "AppDelegate.h"
#import "SettingsController.h"

@interface AppDelegate ()

@property (retain) IBOutlet NSWindow *window;

@property (retain) NSMutableArray* filterList;

@property NSInteger filterListMode;
@end

@implementation AppDelegate

-(instancetype)init
{
	self = [super init];
	
	self.filterList = [NSMutableArray arrayWithCapacity:16];
	
	return self;
}

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
	// Insert code here to initialize your application
	
	SettingsController* del = [SettingsController alloc];
	del = [del initWithWindowNibName:@"Settings"];
	[del retain];
	
	NSLog(@"%@", [del window]);
	
	[del showWindow:nil];
	
	[[[NSWorkspace sharedWorkspace] notificationCenter] addObserver:self selector:@selector(appActivated:) name:NSWorkspaceDidActivateApplicationNotification object:nil];

	//[NSUserDefaults standardUserDefaults] objectForKey:@"";
	[[NSUserDefaultsController sharedUserDefaultsController] addObserver:self
															  forKeyPath:@"FilterListMode"
																 options:NSKeyValueObservingOptionNew
																 context:NULL];
	
	NSRunningApplication* frontApp = [[NSWorkspace sharedWorkspace] frontmostApplication];
	NSLog(@"RunningApp = %@", frontApp);

}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
	// Insert code here to tear down your application
}

- (void) appActivated:(NSNotification*) noti
{
	NSRunningApplication* app =  noti.userInfo[NSWorkspaceApplicationKey];
	NSString* name = app.bundleURL.pathComponents.lastObject;
	
	NSLog(@"appActivated! %@", name);
	
	[self.filterList addObject:name];
	
	NSLog(@"%d", [self.filterList containsObject:@"QQ.app"]);
}

- (void)observeValueForKeyPath:(NSString *)keyPath ofObject:(id)object change:(NSDictionary *)change context:(void *)context
{
	if (context == 0) {
		NSNumber* newValue = [change objectForKey:NSKeyValueChangeNewKey];
		bool val = newValue.intValue;
		
	} else {
		[super observeValueForKeyPath:keyPath ofObject:object change:change context:context];
	}
}

@end
