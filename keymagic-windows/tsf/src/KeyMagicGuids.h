#ifndef KEYMAGIC_GUIDS_H
#define KEYMAGIC_GUIDS_H

#include <windows.h>
#include <initguid.h>

// KeyMagic Text Service CLSID
// {12345678-1234-1234-1234-123456789ABC}
DEFINE_GUID(CLSID_KeyMagicTextService, 
    0x12345678, 0x1234, 0x1234, 0x12, 0x34, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc);

// KeyMagic Profile GUID
// {87654321-4321-4321-4321-CBA987654321}
DEFINE_GUID(GUID_KeyMagicProfile,
    0x87654321, 0x4321, 0x4321, 0x43, 0x21, 0xcb, 0xa9, 0x87, 0x65, 0x43, 0x21);

// KeyMagic Language Bar Button GUID
// {F3BA9079-6C7E-11E0-B278-00215C6A7F0C}
DEFINE_GUID(GUID_KeyMagicLangBarButton,
    0xf3ba9079, 0x6c7e, 0x11e0, 0xb2, 0x78, 0x00, 0x21, 0x5c, 0x6a, 0x7f, 0x0c);

// Display Attribute GUID for composing text
// {F3BA907A-6C7E-11E0-B278-00215C6A7F0C}
DEFINE_GUID(GUID_KeyMagicDisplayAttributeInput,
    0xf3ba907a, 0x6c7e, 0x11e0, 0xb2, 0x78, 0x00, 0x21, 0x5c, 0x6a, 0x7f, 0x0c);

#endif // KEYMAGIC_GUIDS_H