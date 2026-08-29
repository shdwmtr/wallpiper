# Command API

## Usage

```
$ wallpiperctl <command>
```

## Daemon Commands

* `debug-off` toggle debug menu on wallpaper renderer off
* `debug-on` toggle debug menu on wallpaper renderer on

## Wallpaper Engine Commands

* `pause`
* `play`
* `stop`
* `next`
* `prev`
* `reset`
* `mute`
* `unmute`
* `volume <0-100>`
* `set <path|workshop-id> [monitor; int; 0-indexed]`
* `prop <path|workshop-id> <name> <value> [monitor; int; 0-indexed]`
* `list`

## Standalone

* `check-config`
