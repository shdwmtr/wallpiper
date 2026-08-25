## [1.2.2](https://github.com/shdwmtr/wallpiper/compare/v1.2.1...v1.2.2) (2026-08-25)


### Bug Fixes

* Fix tray rendering issues, and tray being duplicated on GNOME and KDE. Closes [#8](https://github.com/shdwmtr/wallpiper/issues/8) ([75ac9fb](https://github.com/shdwmtr/wallpiper/commit/75ac9fb79d33d42c05c620cc90628872219a7d7f))

## [1.2.1](https://github.com/shdwmtr/wallpiper/compare/v1.2.0...v1.2.1) (2026-08-25)


### Bug Fixes

* fix wallpiper failing to IPC with external portals (GNOME, KDE) ([d690ead](https://github.com/shdwmtr/wallpiper/commit/d690ead69de98203a94cdd1066ced9545ac61c75))

# [1.2.0](https://github.com/shdwmtr/wallpiper/compare/v1.1.4...v1.2.0) (2026-08-25)


### Features

* Disable hardware acceleration by default (still able to turn back on in WE Settings) ([8768ef0](https://github.com/shdwmtr/wallpiper/commit/8768ef032f87494d14572321fbce247f707b53ba))

## [1.1.4](https://github.com/shdwmtr/wallpiper/compare/v1.1.3...v1.1.4) (2026-08-25)


### Bug Fixes

* fix fsutil overwriting config ([a6cba39](https://github.com/shdwmtr/wallpiper/commit/a6cba3942fe270af412b00b26ad335f86385ff13))

## [1.1.3](https://github.com/shdwmtr/wallpiper/compare/v1.1.2...v1.1.3) (2026-08-25)


### Bug Fixes

* fix wallpaper engines broken default installation. Closes [#14](https://github.com/shdwmtr/wallpiper/issues/14) ([c00bf2d](https://github.com/shdwmtr/wallpiper/commit/c00bf2db3e4ab20b69c1205e7e8c6861c8cdeefa))

## [1.1.2](https://github.com/shdwmtr/wallpiper/compare/v1.1.1...v1.1.2) (2026-08-25)


### Bug Fixes

* stop targeting arbitrary python3 processes as old proton wrappers. Closes [#13](https://github.com/shdwmtr/wallpiper/issues/13) ([608610d](https://github.com/shdwmtr/wallpiper/commit/608610d6518df7a7a393c25583f82c8636a4ab6e))

## [1.1.1](https://github.com/shdwmtr/wallpiper/compare/v1.1.0...v1.1.1) (2026-08-24)


### Bug Fixes

* fix build system. Closes [#12](https://github.com/shdwmtr/wallpiper/issues/12) ([53f7b0b](https://github.com/shdwmtr/wallpiper/commit/53f7b0b5f350fce4c23eaedd52705ad891c5bb94))

# [1.1.0](https://github.com/shdwmtr/wallpiper/compare/v1.0.0...v1.1.0) (2026-08-24)


### Features

* **wallpiperctl:** proper CLI support, check README for documentation. Closes [#9](https://github.com/shdwmtr/wallpiper/issues/9) ([ca3810d](https://github.com/shdwmtr/wallpiper/commit/ca3810d41dc6cd41845699a27e566fa9fea37cb5))


### Performance Improvements

* remove redundant portal side monitor geometry setup ([81990f5](https://github.com/shdwmtr/wallpiper/commit/81990f513a6abaea9b0d6737d8438c202c8f9bc2))

# 1.0.0 (2026-08-24)


### Bug Fixes

* Fix gitignore ignoring source files ([1eb8e56](https://github.com/shdwmtr/wallpiper/commit/1eb8e56ae4d009c006a4bb0c953b6c879777c16b))
* fix tray menu ([64a75b2](https://github.com/shdwmtr/wallpiper/commit/64a75b2a29eb9917c79719d2f7a3b643b23eb20f))
* fix web wallpapers ([63fe4ca](https://github.com/shdwmtr/wallpiper/commit/63fe4ca5e98bacc8a4827222185c43ced21836a7))
* install KDE plugin to proper path ([285ab55](https://github.com/shdwmtr/wallpiper/commit/285ab55a30f537aaf0f2f5ea69b4b0c0ef1c9f82))
* properly reap child process ([53fb14f](https://github.com/shdwmtr/wallpiper/commit/53fb14f5fd08b8187f20a33d3f773c468e7ad09f))


### Features

* add sway support ([0cf8603](https://github.com/shdwmtr/wallpiper/commit/0cf86038f09999320c10db5dbf100fcdabaf2100))
* add wallpiper to AUR ([cddd4d9](https://github.com/shdwmtr/wallpiper/commit/cddd4d9d0880d374b74572c61613c63da5cef0b1))
* full implementation ([ae5124f](https://github.com/shdwmtr/wallpiper/commit/ae5124f2b87798030bac7a036a199040e34a5b5f))
* i3wm portal support ([6244e7f](https://github.com/shdwmtr/wallpiper/commit/6244e7f0427a972f7c5f6f193786aa9fbe15146a))
* list-wallpapers and list-properties support ([d146789](https://github.com/shdwmtr/wallpiper/commit/d1467896bfdeb0b110ad666d4dff2ce3aeee91f1))
* multi-monitor wallpaper support across all portals ([dc38d86](https://github.com/shdwmtr/wallpiper/commit/dc38d866fa2ffdd388034b53aefa29996a3a924d))
* properly scale the wallpaperui picker on hyprland, sway ([6c1edf3](https://github.com/shdwmtr/wallpiper/commit/6c1edf317ed5351b7bd8823f1317a50b08775a2c))
* reduce binary sizes ([d2fce2d](https://github.com/shdwmtr/wallpiper/commit/d2fce2d700e6ffa6b41c0a5bbecf482bddbf3ee3))
* support for COSMIC ([ee9feb8](https://github.com/shdwmtr/wallpiper/commit/ee9feb88cf1707cbcf2b8f08e767056139328b2d))
* Support for GNOME's mutter compositor ([7d4eec6](https://github.com/shdwmtr/wallpiper/commit/7d4eec6eb6a4e8dfed9ff6a9082b853be3b513f0))
* **wallpiper-portal:** Support for KDE Plasma natively with a wallpaper plugin ([96b16ef](https://github.com/shdwmtr/wallpiper/commit/96b16ef9eef9fb29355dc6e384ad20772dba4eb4))
