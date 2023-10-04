# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased]

### Added

- Ship controls object, which is the target of the "launch ship" command
- "refuel ship" command
- Food ration item
- Defined prices for more items types, allowing more items as stock in stores or to be sold to stores
- Command suggestions on click for location choice
- Crew members remain as corpses after dying

### Changed

- Save format version 2.0, making it incompatible with previous save files
- Reduced healing when moving from one location to the next
- Healing between locations now requires a food ration 
- Increase the price of swords
- The cut variant for doors is now handled through texture layers in the same texture data file instead of with separate texture data files
- Object textures are now loaded lazily instead of all at once
- Background types are no longer hardcoded, making it possible to introduce new types with just the background data file and location files
- "use fuel can" is now equivalent to "refuel ship" instead of "launch ship"
- Updated items in all locations to add some food availability

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
