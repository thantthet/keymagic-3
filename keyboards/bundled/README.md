# Bundled Keyboards

This directory contains keyboard files that are bundled with KeyMagic releases for all platforms.

## Current Keyboards

- **Malayalam Mozhi.km2** - Malayalam input method using Mozhi scheme
- **MyanSan.km2** - မြန်စံ Myanmar unicode layout
- **Pyidaungsu MM.km2** - ပြည်ထောင်စု Myanmar unicode keyboard layout
- **ZawCode.km2** - ဇော်ကုဒ် Myanmar unicode keyboard layout

## Requirements

- Files must have `.km2` extension
- Files should be tested and production-ready
- Include keyboards for major languages/scripts

## Build Process

These keyboards are automatically included during the build process for:
- Windows installer (via Inno Setup)
- macOS app bundle (in Resources)
- Linux packages (deb/rpm)