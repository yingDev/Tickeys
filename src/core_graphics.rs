#![macro_use]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]

extern crate libc;
extern crate cocoa;

use self::libc::{c_void};
use core_foundation::CFMachPortRef;

#[repr(u32)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub enum CGEventTapLocation 
{
   kCGHIDEventTap = 0,
   kCGSessionEventTap,
   kCGAnnotatedSessionEventTap 
}

#[repr(u32)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum CGEventTapPlacement 
{
   kCGHeadInsertEventTap = 0,
   kCGTailAppendEventTap 
}

#[repr(u32)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum CGEventTapOptions 
{
   kCGEventTapOptionDefault = 0x00000000,
   kCGEventTapOptionListenOnly = 0x00000001 
}

#[repr(u32)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum CGEventType
{
  /* The null event. */
  kCGEventNull = 0,//NX_NULLEVENT,

  /* Mouse events. */
  kCGEventLeftMouseDown = 1,//NX_LMOUSEDOWN,
  kCGEventLeftMouseUp = 2,//NX_LMOUSEUP,
  kCGEventRightMouseDown = 3,//NX_RMOUSEDOWN,
  kCGEventRightMouseUp = 4,//NX_RMOUSEUP,
  kCGEventMouseMoved = 5,//NX_MOUSEMOVED,
  kCGEventLeftMouseDragged = 6,//NX_LMOUSEDRAGGED,
  kCGEventRightMouseDragged = 7,//NX_RMOUSEDRAGGED,

  /* Keyboard events. */
  kCGEventKeyDown = 10,//NX_KEYDOWN,
  kCGEventKeyUp = 11,//NX_KEYUP,
  kCGEventFlagsChanged = 12,//NX_FLAGSCHANGED,

  /* Specialized control devices. */
  kCGEventScrollWheel = 22,//NX_SCROLLWHEELMOVED,
  kCGEventTabletPointer = 23,//NX_TABLETPOINTER,
  kCGEventTabletProximity = 24,//NX_TABLETPROXIMITY,
  kCGEventOtherMouseDown = 25,//NX_OMOUSEDOWN,
  kCGEventOtherMouseUp = 26,//NX_OMOUSEUP,
  kCGEventOtherMouseDragged = 27,//NX_OMOUSEDRAGGED,

  /* Out of band event types. These are delivered to the event tap callback
     to notify it of unusual conditions that disable the event tap. */
  kCGEventTapDisabledByTimeout = 0xFFFFFFFE,
  kCGEventTapDisabledByUserInput = 0xFFFFFFFF
}

#[repr(i32)]
pub enum CGWindowLevelKey 
{
    kCGBaseWindowLevelKey = 0,
    kCGMinimumWindowLevelKey,
    kCGDesktopWindowLevelKey,
    kCGBackstopMenuLevelKey,
    kCGNormalWindowLevelKey,
    kCGFloatingWindowLevelKey,
    kCGTornOffMenuWindowLevelKey,
    kCGDockWindowLevelKey,
    kCGMainMenuWindowLevelKey,
    kCGStatusWindowLevelKey,
    kCGModalPanelWindowLevelKey,
    kCGPopUpMenuWindowLevelKey,
    kCGDraggingWindowLevelKey,
    kCGScreenSaverWindowLevelKey,
    kCGMaximumWindowLevelKey,
    kCGOverlayWindowLevelKey,
    kCGHelpWindowLevelKey,
    kCGUtilityWindowLevelKey,
    kCGDesktopIconWindowLevelKey,
    kCGCursorWindowLevelKey,
    kCGAssistiveTechHighWindowLevelKey,
    kCGNumberOfWindowLevelKeys  /* Must be last. */
}

#[repr(u32)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum CGEventField 
{
  /* Key to access an integer field that contains the mouse button event
     number. Matching mouse-down and mouse-up events will have the same
     event number. */
  kCGMouseEventNumber = 0,

  /* Key to access an integer field that contains the mouse button click
  state. A click state of 1 represents a single click. A click state of 2
  represents a double-click. A click state of 3 represents a
  triple-click. */
  kCGMouseEventClickState = 1,

  /* Key to access a double field that contains the mouse button pressure.
     The pressure value may range from 0 to 1, with 0 representing the mouse
     being up. This value is commonly set by tablet pens mimicking a
     mouse. */
  kCGMouseEventPressure = 2,

  /* Key to access an integer field that contains the mouse button
     number. */
  kCGMouseEventButtonNumber = 3,

  /* Key to access an integer field that contains the horizontal mouse delta
     since the last mouse movement event. */
  kCGMouseEventDeltaX = 4,

  /* Key to access an integer field that contains the vertical mouse delta
     since the last mouse movement event. */
  kCGMouseEventDeltaY = 5,

  /* Key to access an integer field. The value is non-zero if the event
     should be ignored by the Inkwell subsystem. */
  kCGMouseEventInstantMouser = 6,

  /* Key to access an integer field that encodes the mouse event subtype as
     a `kCFNumberIntType'. */
  kCGMouseEventSubtype = 7,

  /* Key to access an integer field, non-zero when this is an autorepeat of
     a key-down, and zero otherwise. */
  kCGKeyboardEventAutorepeat = 8,

  /* Key to access an integer field that contains the virtual keycode of the
     key-down or key-up event. */
  kCGKeyboardEventKeycode = 9,

  /* Key to access an integer field that contains the keyboard type
     identifier. */
  kCGKeyboardEventKeyboardType = 10,

  /* Key to access an integer field that contains scrolling data. This field
     typically contains the change in vertical position since the last
     scrolling event from a Mighty Mouse scroller or a single-wheel mouse
     scroller. */
  kCGScrollWheelEventDeltaAxis1 = 11,

  /* Key to access an integer field that contains scrolling data. This field
     typically contains the change in horizontal position since the last
     scrolling event from a Mighty Mouse scroller. */
  kCGScrollWheelEventDeltaAxis2 = 12,

  /* This field is not used. */
  kCGScrollWheelEventDeltaAxis3 = 13,

  /* Key to access a field that contains scrolling data. The scrolling data
     represents a line-based or pixel-based change in vertical position
     since the last scrolling event from a Mighty Mouse scroller or a
     single-wheel mouse scroller. The scrolling data uses a fixed-point
     16.16 signed integer format. If this key is passed to
     `CGEventGetDoubleValueField', the fixed-point value is converted to a
     double value. */
  kCGScrollWheelEventFixedPtDeltaAxis1 = 93,

  /* Key to access a field that contains scrolling data. The scrolling data
     represents a line-based or pixel-based change in horizontal position
     since the last scrolling event from a Mighty Mouse scroller. The
     scrolling data uses a fixed-point 16.16 signed integer format. If this
     key is passed to `CGEventGetDoubleValueField', the fixed-point value is
     converted to a double value. */
  kCGScrollWheelEventFixedPtDeltaAxis2 = 94,

  /* This field is not used. */
  kCGScrollWheelEventFixedPtDeltaAxis3 = 95,

  /* Key to access an integer field that contains pixel-based scrolling
     data. The scrolling data represents the change in vertical position
     since the last scrolling event from a Mighty Mouse scroller or a
     single-wheel mouse scroller. */
  kCGScrollWheelEventPointDeltaAxis1 = 96,

  /* Key to access an integer field that contains pixel-based scrolling
     data. The scrolling data represents the change in horizontal position
     since the last scrolling event from a Mighty Mouse scroller. */
  kCGScrollWheelEventPointDeltaAxis2 = 97,

  /* This field is not used. */
  kCGScrollWheelEventPointDeltaAxis3 = 98,
    
  /*  */
  kCGScrollWheelEventScrollPhase = 99,
    
  /* rdar://11259169 */
  kCGScrollWheelEventScrollCount = 100,
    
  kCGScrollWheelEventMomentumPhase = 123,
    
  /* Key to access an integer field that indicates whether the event should
     be ignored by the Inkwell subsystem. If the value is non-zero, the
     event should be ignored. */
  kCGScrollWheelEventInstantMouser = 14,

  /* Key to access an integer field that contains the absolute X coordinate
     in tablet space at full tablet resolution. */
  kCGTabletEventPointX = 15,

  /* Key to access an integer field that contains the absolute Y coordinate
     in tablet space at full tablet resolution. */
  kCGTabletEventPointY = 16,

  /* Key to access an integer field that contains the absolute Z coordinate
     in tablet space at full tablet resolution. */
  kCGTabletEventPointZ = 17,

  /* Key to access an integer field that contains the tablet button state.
     Bit 0 is the first button, and a set bit represents a closed or pressed
     button. Up to 16 buttons are supported. */
  kCGTabletEventPointButtons = 18,

  /* Key to access a double field that contains the tablet pen pressure. A
     value of 0.0 represents no pressure, and 1.0 represents maximum
     pressure. */
  kCGTabletEventPointPressure = 19,

  /* Key to access a double field that contains the horizontal tablet pen
     tilt. A value of 0 represents no tilt, and 1 represents maximum tilt. */
  kCGTabletEventTiltX = 20,

  /* Key to access a double field that contains the vertical tablet pen
     tilt. A value of 0 represents no tilt, and 1 represents maximum tilt. */
  kCGTabletEventTiltY = 21,

  /* Key to access a double field that contains the tablet pen rotation. */
  kCGTabletEventRotation = 22,

  /* Key to access a double field that contains the tangential pressure on
     the device. A value of 0.0 represents no pressure, and 1.0 represents
     maximum pressure. */
  kCGTabletEventTangentialPressure = 23,

  /* Key to access an integer field that contains the system-assigned unique
     device ID. */
  kCGTabletEventDeviceID = 24,

  /* Key to access an integer field that contains a vendor-specified value. */
  kCGTabletEventVendor1 = 25,

  /* Key to access an integer field that contains a vendor-specified value. */
  kCGTabletEventVendor2 = 26,

  /* Key to access an integer field that contains a vendor-specified value. */
  kCGTabletEventVendor3 = 27,

  /* Key to access an integer field that contains the vendor-defined ID,
     typically the USB vendor ID. */
  kCGTabletProximityEventVendorID = 28,

  /* Key to access an integer field that contains the vendor-defined tablet
     ID, typically the USB product ID. */
  kCGTabletProximityEventTabletID = 29,

  /* Key to access an integer field that contains the vendor-defined ID of
     the pointing device. */
  kCGTabletProximityEventPointerID = 30,

  /* Key to access an integer field that contains the system-assigned device
     ID. */
  kCGTabletProximityEventDeviceID = 31,

  /* Key to access an integer field that contains the system-assigned unique
     tablet ID. */
  kCGTabletProximityEventSystemTabletID = 32,

  /* Key to access an integer field that contains the vendor-assigned
     pointer type. */
  kCGTabletProximityEventVendorPointerType = 33,

  /* Key to access an integer field that contains the vendor-defined pointer
     serial number. */
  kCGTabletProximityEventVendorPointerSerialNumber = 34,

  /* Key to access an integer field that contains the vendor-defined unique
     ID. */
  kCGTabletProximityEventVendorUniqueID = 35,

  /* Key to access an integer field that contains the device capabilities
     mask. */
  kCGTabletProximityEventCapabilityMask = 36,

  /* Key to access an integer field that contains the pointer type. */
  kCGTabletProximityEventPointerType = 37,

  /* Key to access an integer field that indicates whether the pen is in
     proximity to the tablet. The value is non-zero if the pen is in
     proximity to the tablet and zero when leaving the tablet. */
  kCGTabletProximityEventEnterProximity = 38,

  /* Key to access a field that contains the event target process serial
     number. The value is a 64-bit value. */
  kCGEventTargetProcessSerialNumber = 39,

  /* Key to access a field that contains the event target Unix process ID. */
  kCGEventTargetUnixProcessID = 40,

  /* Key to access a field that contains the event source Unix process ID. */
  kCGEventSourceUnixProcessID = 41,

  /* Key to access a field that contains the event source user-supplied
     data, up to 64 bits. */
  kCGEventSourceUserData = 42,

  /* Key to access a field that contains the event source Unix effective
     UID. */
  kCGEventSourceUserID = 43,

  /* Key to access a field that contains the event source Unix effective
     GID. */
  kCGEventSourceGroupID = 44,

  /* Key to access a field that contains the event source state ID used to
     create this event. */
  kCGEventSourceStateID = 45,
    
  /* Key to access an integer field that indicates whether a scrolling event
     contains continuous, pixel-based scrolling data. The value is non-zero
     when the scrolling data is pixel-based and zero when the scrolling data
     is line-based. */
  kCGScrollWheelEventIsContinuous = 88,
  
  /* Added in 10.5; made public in 10.7 */
  kCGMouseEventWindowUnderMousePointer = 91,
  kCGMouseEventWindowUnderMousePointerThatCanHandleThisEvent = 92
}

pub type CGEventMask = u64;
pub type CGEventTapProxy = *mut c_void;//opaque
pub type CGEventRef = *mut c_void;//opaque
pub type CGEventTapCallBack = extern fn(proxy: CGEventTapProxy, etype: CGEventType, event: CGEventRef, refcon: *mut c_void) -> CGEventRef;

#[macro_export]
macro_rules! CGEventMaskBit
{
    ($eventType: expr) => { 1 << ($eventType as CGEventMask)};
}

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
pub const kCGEventMaskForAllEvents:u64 = !0;

#[link(name = "CoreGraphics", kind = "framework")]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
extern "system"
{
	pub fn CGEventTapCreate (
		   tap: CGEventTapLocation,
		   place: CGEventTapPlacement,
		   options: CGEventTapOptions,
		   eventsOfInterest: CGEventMask,
		   callback: CGEventTapCallBack,
		   userInfo: *mut c_void) -> CFMachPortRef;

	pub fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;

	pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

	pub fn CGEventTapIsEnabled(tap: CFMachPortRef) -> bool;

  pub fn CGWindowLevelForKey(key: CGWindowLevelKey) -> i32;

}