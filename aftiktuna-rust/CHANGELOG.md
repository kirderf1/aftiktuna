# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased]

### Added

- New data file `character_profiles.json`, used for randomly-selected characters
- New aftik colors and character profiles

### Changed

- Recruitable characters and crew members in json definitions can now be changed to pick a character profile at random

### Fixed

- Save file is now removed when reaching the victory/game over frame
- After victory/game over, the game returns to the main menu
- "Load Game" button is no longer shown when the save file is absent

### Contributors for this release

- kirderf1


## [v0.8.0] - 2024-06-07

### Added

- New symbol types "aftik_corpse" and "inanimate" for creating locations
- Luck stat, which currently influences hit/dodge rolls
- Added character traits: "Big Eater", "Fast Healer", "Fragile", "Good Dodger"
- Creature attributes, which slightly adjust their stats and is chosen at random by default
- `crew_data.json` data file, which defines the crew used when creating a new game
- It is now possible to create backgrounds with a parallax effect in `backgrounds.json`

### Changed

- Save format version 3.0, making it incompatible with previous save files
- Updated readme.txt
- Expanded shopkeeper definition to allow setting a custom price
- Shops can now have a limited quantity of items in stock,
 and the village store has been updated accordingly
- Updated outdoor background textures, and added a parallax effect to these
- Changed the format of the "backgrounds.json" data file
- Non-controlled crew members now use held medkits on their own when in low health
- Crew members now object to selling fuel cans that are needed to refuel the ship
- Wounded character portrait in dialogue
- Health value can be specified for placed creatures when creating locations
- Initial aggressiveness can be specified for individual creatures when creating locations
- Edits to the eyesaur forest location
- Tweaked how attacker agility was factored into dodge rolls
- Three or more alive creatures are no longer allowed to stand on the same space

### Contributors for this release

- kirderf1


## [v0.7.2] - 2024-05-21

### Added

- Main Menu
- Loot symbol for location definitions which spawns an item from a builtin loot table
- Builtin loot tables "regular" and "valuable"
- "check <item>" command, which gives information on an item
- "go to ship" command
- "tell _ to wait at ship" command
- New alternative for fortuna encounter
- Very basic texture loading screen

### Changed

- Save format version 2.1
- The displayed name of dead creatures is now prefixed as "Corpse of ..."
- Non-crew creatures now remain as corpses when they die
- All creatures now have a wounded texture variant
- Locations now use random loot
- Other minor adjustments to locations
- Reduced value of blowtorches
- Tweaked some command parsing error messages
- Encountering a foe now comes with a message and the foe turning to face the character
- Certain creatures will now not start attacking immediately
- Aftik colors are no longer hardcoded, and their color values are now defined in "aftik_colors.json"
- Some locations now have multiple possible landing spots
- Texture loading errors are now displayed in the game window
- All command suggestions except for "buy" now show up when clicking anywhere in the store

### Fixed

- Reassign symbols on conflict between labels in the textual version of the game

### Contributors for this release

- kirderf1


## [v0.6.0] - 2023-10-05

### Added

- Food ration item
- Crew members remain as corpses after dying
- Ship controls object, which is the target of the "launch ship" command
- "refuel ship" command
- Command suggestions on click for location choice
- Command suggestions on click for "give", "wield", "sell" and "sell all"
- Defined prices for more items types, allowing more items as stock in stores or to be sold to stores

### Changed

- Save format version 2.0, making it incompatible with previous save files
- "use fuel can" is now equivalent to "refuel ship" instead of "launch ship"
- Reduced healing when moving from one location to the next
- Healing between locations now requires a food ration
- Updated items in all locations to add some food availability
- Increase the price of swords
- Background types are no longer hardcoded, making it possible to introduce new types with just the background data file and location files
- Object textures are now loaded lazily instead of all at once
- The cut variant for doors is now handled through texture layers in the same texture data file instead of with separate texture data files

### Fixed

- Clamp tooltip position so that tooltips does not extend outside the right edge of the game window

### Contributors for this release

- kirderf1


## [v0.5.0] - 2023-09-09

### Added

- Dialogue frames with "recruit" and "give" commands
- Dialogue (kind of) with "trade" command
- New commands "talk to <target>", "tell <target> to wait", "tell <target> to follow"
- The other crew member may help force open a door if they have the right tool for it
- Add warning message when ship is refueled without everyone on board
- Add message when a crew member is left behind after leaving a location

### Changed

- Save format version 1.1
- Background render data now includes data on background used for dialogue frames
- Update aftik textures

### Contributors for this release

- kirderf1


## [v0.4.0] - 2023-08-31

### Added

- New creatures Scarvies and Voracious frogs
- Textured store graphics with command suggestions on click to replace the placeholder store graphic
- New types of symbol data for location data files
- Base symbol palette data file to replace the remaining hardcoded symbols
- Data files for texture render data that was previously hardcoded
- Single data file for background render data that was previously hardcoded
- Flag for executable `--new-game`

### Changed

- Save format version 1.0, making it incompatible with previous save files
- Adjustments to existing locations
- Adjusted various textures, including some outdoor background textures and creature texture sizes
- Controlled character now drawn over other crew members in the same position
- One frame may now show events across multiple game ticks

### Contributors for this release

- kirderf1


## [v0.3.0] - 2023-08-23

### Added

- Save files starting with format 0.0 (intention is to update the first number when breaking backwards compatibility, and update the second number when breaking forwards compatibility)
- The game is automatically saved when closed, and automatically loaded when opened
- Flag for executable `--disable-autosave` that disables save on close
- Choosable command suggestions when clicking game objects

### Changed

- Minor tweaks to text box/tool tip rendering
- The win or lose end frame is now a blank screen
- Tweak some edge cases to attacks
- Separate events after an action failure as a new frame
- Updates to readme file

### Fixed

- Mouse tooltip is now drawn over the text box

### Contributors for this release

- kirderf1


## [v0.2.0] - 2023-08-14

### Added

- Final location with a fortuna chest
- New win condition by opening the fortuna chest with a new command
- Simple introduction frame at start of game
- Status messages relating to stamina
- Cut door texture variant for shack

### Changed

- `locations.json` format now contains a list of possible final locations
- Tweaked some messages
- Adjusted how the controlled character may wield an item

### Contributors for this release

- kirderf1


## [v0.1.4] - 2023-08-09

### Added

- Arrow icons on screen in directions that the camera can be dragged
- Door texture variant for doors cut with a blowtorch

### Changed

- "rest" command now considers all crew members in the area
- Changed message for when a creature blocks the path of an action
- Texture adjustments
- Input field focus improvements

### Contributors for this release

- kirderf1


## [v0.1.2] - 2023-08-07

### Added

- Render items wielded by characters

### Changed

- More action conditions are now checked during user input phase
- The controlled character may wait for another crew member to wield a given weapon

### Contributors for this release

- kirderf1


## [v0.1.1] - 2023-07-24

### Added

- Placeholder graphic when accessing a store

### Changed

- Updated notes in readme file

### Contributors for this release

- kirderf1


## [v0.1.0] - 2023-07-18

- Initial release

### Contributors for this release

- kirderf1
