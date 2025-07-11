#ifndef KEYMAGIC_GUIDS_H
#define KEYMAGIC_GUIDS_H

#include <windows.h>
#include <initguid.h>

// KeyMagic Text Service CLSID
// {094A562B-D08B-4CAF-8E95-8F8031CFD24C}
DEFINE_GUID(CLSID_KeyMagicTextService, 
    0x094a562b, 0xd08b, 0x4caf, 0x8e, 0x95, 0x8f, 0x80, 0x31, 0xcf, 0xd2, 0x4c);

// KeyMagic Profile GUID
// {C29D9340-87AA-4149-A1CE-F6ACAA8AF30B}
DEFINE_GUID(GUID_KeyMagicProfile,
    0xc29d9340, 0x87aa, 0x4149, 0xa1, 0xce, 0xf6, 0xac, 0xaa, 0x8a, 0xf3, 0x0b);

// KeyMagic Language Bar Button GUID
// {9756F03C-080F-4692-B779-25DBEC1FE48F}
DEFINE_GUID(GUID_KeyMagicLangBarButton,
    0x9756f03c, 0x080f, 0x4692, 0xb7, 0x79, 0x25, 0xdb, 0xec, 0x1f, 0xe4, 0x8f);

// Display Attribute GUID for composing text
// {2839B100-4CB8-4079-B44B-8032D4C70342}
DEFINE_GUID(GUID_KeyMagicDisplayAttributeInput,
    0x2839b100, 0x4cb8, 0x4079, 0xb4, 0x4b, 0x80, 0x32, 0xd4, 0xc7, 0x03, 0x42);

#endif // KEYMAGIC_GUIDS_H