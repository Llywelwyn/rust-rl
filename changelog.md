## v0.1.4
### added
- **overmap**: bare, but exists. player now starts on the overworld, and can move to local maps (like the old starting town) via >. can leave local maps back to the overmap by walking out of the map boundaries.
- **full keyboard support**: examining and targeting can now be done via keyboard only
- **a config file** read at runtime, unfortunately not compatible with WASM builds yet
- **morgue files**: y/n prompt to write a morgue file on death to /morgue/foo.txt, or to the console for WASM builds
- **dungeon features**: just the basics so far. a grassy, forested treant room, some barracks, etc.
- **named maps**: "Town", "Dungeon"
- **map messages/hints**: "You hear <...>."
### changed
- **colour offsets** are now per-tile (and per-theme) instead of +-% globally. i.e. varying fg/bg offset on a per-tiletype basis
- **chatlog colours** are now consistent
### fixed
- negative starting mana
- status effects only ticking if mob turn aligned with turnclock turn
- map params not being saved on map transition
- mob turns not awaiting the particle queue (mobs moving around mid-animation)
- mobs not re-pathing if their path was blocked, causing traffic jams
