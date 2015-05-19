//
//  SettingsDelegate.m
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import "SettingsDelegate.h"

@implementation SettingsDelegate

-(instancetype)init
{
    self = [super init];
	
	self.user_data = (void*)123;
	
	return self;
}

- (IBAction)quit:(id)sender {
	[NSApp terminate:nil];
}
- (IBAction)follow_link:(id)sender {
}
- (IBAction)value_changed:(id)sender
{
	[[NSUserDefaults standardUserDefaults] setObject:@"shit" forKey:@"shit"];
}

-(void)windowWillClose:(NSNotification *)notification
{
	NSLog(@"%s", [@"shit" UTF8String]);
}

@end
