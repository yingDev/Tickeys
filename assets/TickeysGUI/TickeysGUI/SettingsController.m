//
//  SettingsDelegate.m
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import "SettingsController.h"

@implementation SettingsController

-(instancetype)init
{
    self = [super init];
	
	self.user_data = (void*)123;
	NSLog(@"%@", NSLocalizedString(@"quit", @""));
	
	
	return self;
}

-(void)windowDidLoad
{
	[[self.window standardWindowButton:NSWindowZoomButton] setHidden:YES];
	[[self.window standardWindowButton:NSWindowMiniaturizeButton] setHidden:YES];
	
	self.window.level = CGWindowLevelForKey(kCGFloatingWindowLevelKey); // 5
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

- (IBAction)btnAddClicked:(id)sender
{
	NSOpenPanel* open = [NSOpenPanel openPanel];
	//open.canChooseDirectories = true;
	//open.canChooseFiles = false;
	//open.treatsFilePackagesAsDirectories = false;
	NSMutableArray* allowedTypes = [NSMutableArray arrayWithCapacity:2];
	[allowedTypes addObject:@"app"];
	
	open.allowsMultipleSelection = true;
	
	open.allowedFileTypes = allowedTypes;
	open.directoryURL = [NSURL URLWithString:@"/Applications"];
	
	if ([open runModal] == 1)
	{
		NSArray* urls = open.URLs;
		
		[self.filterListTable reloadData];
		
	}
}

- (IBAction)btnRemoveClicked:(id)sender {
	NSIndexSet* selectedRows = self.filterListTable.selectedRowIndexes;
	[self.filterListTable beginUpdates];
	[self.filterListTable removeRowsAtIndexes:selectedRows withAnimation:0];
	[self.filterListTable endUpdates];
	
	
	
//	[NSUserDefaults standardUserDefaults] setObject:<#(nullable id)#> forKey:<#(nonnull NSString *)#>
}



-(void)windowWillClose:(NSNotification *)notification
{
	NSLog(@"%s", [@"shit" UTF8String]);
	
}

-(NSInteger)numberOfRowsInTableView:(NSTableView *)tableView
{
	return 10;
}

-(id)tableView:(NSTableView *)tableView objectValueForTableColumn:(NSTableColumn *)tableColumn row:(NSInteger)row
{
	return @"hello";
}

-(BOOL)tableView:(NSTableView *)tableView shouldEditTableColumn:(NSTableColumn *)tableColumn row:(NSInteger)row
{
	return NO;
}

@end
