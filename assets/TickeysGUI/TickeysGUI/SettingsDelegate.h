//
//  SettingsDelegate.h
//  TickeysGUI
//
//  Created by Ying on 5/18/15.
//  Copyright (c) 2015 YingDev.com. All rights reserved.
//

#import <Foundation/Foundation.h>
#import <Cocoa/Cocoa.h>

@interface SettingsDelegate : NSObject
@property (assign) IBOutlet NSPopUpButton *popup_audio_scheme;
@property (assign) IBOutlet NSSlider *slide_volume;
@property (assign) IBOutlet NSSlider *slide_pitch;

@end
