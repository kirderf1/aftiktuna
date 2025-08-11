# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).


## [Unreleased]

### Changed

- Save format version 5.0, making it incompatible with previous save files
- Attacks now come in a few different flavors with slightly different properties
- Stun recovery now comes with a message
- New condition for view capturing to limit buildup of messages

### Contributors for this release

- kirderf1


## [v0.12.0] - 2025-08-10

### Added

- Idle animation for all creatures
- New creature Blood mantis
- New indoor decorations
- Ship dialogue
- Dialogue expressions
- Random room furnishing from new data files in `assets/area_furnish/`
- Json location definition `crew_ship.json` for the ship and new symbol types "ship_controls" and "food_deposit"
- Noise message when something happens in a nearby area

### Changed

- Save format version 4.1
- Expanded ship layout from one room to three rooms
- Crew ai improvements for moving back to the ship
- Village buildings are visually larger
- Tweaked Fortuna locations and making them slightly harder
- Slightly tweaked placement and ai of wandering creatures
- Repeated actions such as "take all" and "rest" are now cancelled if the area becomes unsafe
- Fine-tuned conditions for using "tell \<character\> to wait" and "tell \<character\> to wait at ship"
- Reworked description for attack actions to be more descriptive
- Made tweaks to the description for enter actions to be more detailed
- New condition for view capturing to help show the state of a newly entered area before it changes
- Objects and creatures now use either "a" or "an" indefinite articles
- Model format changes:
  - "fixed" layer property replaced by "fixed_orientation" model property
  - Flipped the y-axis of "y_offset" and "wield_offset", which now matches the y-axis of group placement and background layer offsets
  - "y_offset" layer property replaced by "offset" layer property with labeled values and optional animation between two values
  - "wield_offset" has changed format to use labeled values
  - New layer properties for rotation animation and rotation-anchor
  - Colored texture layers are now defined differently
  - Bone-like layers with child layers can now be defined, applying its offset and rotation to all child layers

### Fixed

- Fixed rendering on monitors with changed Windows scale factor
- Fixed item matching for the "sell \<number\> \<items\>" and "sell all \<items\>" commands

### Contributors for this release

- kirderf1


## [v0.11.0] - 2025-07-13

### Added

- Creatures can optionally be wandering
- When encountering the ship, wandering creatures may stay around and examine the ship
- Visual darkness effect for areas with "darkness" setting, used in the abandoned facility locations
- Creature taming mechanics with new commands "tame" and "name", currently only applying to lone scarvies and eyesaurs

### Changed

- Save format version 4.0, making it incompatible with previous save files
- Noticeable rework of most locations
- Village store now stocks a blowtorch
- Increased health of everything by 50%
- Tweaked stamina stat, making stamina last longer but also making it not recover while moving
- Reworked field backgrounds with manually placed tree objects
- Reworked background grass details
- Tweak paths placed in front of doors to visually connect to the doors
- Changed location selection to also give a short description of the options
- New model field "z_offset" that helps influence the rendering order
- New model fields "has_x_displacement" and "z_displacement" that replaces "mounted"
- New door pair field "is_cut" to set doors in an already-cut visual state
- Extra background layers can now be specified in the location area definition
- "background_offset" in area data can now be negative

### Fixed

- Fixed potential issue in command parsing with special symbols

### Removed

- Removed the "keycard" item and the "locked" block type

### Contributors for this release

- kirderf1


## [v0.10.0] - 2025-06-07

### Added

- "exit game" command, which may save the game and then return to the main menu
- Added stun effect, which may be received when attacked by a bat
- Additional decorative objects: mossy rock, tree stub, table
- Rendering-relevant system for grouping multiple objects of the same kind in predetermined manner, implemented for food rations and ancient coins

### Changed

- Save format version 3.2
- Various text improvements
  - Certain event-describing sentences will be merged together
  - When searching a container, items of the same kind will now be counted together in the resulting message
  - Some messages now use numbers in words instead of numerals
  - Fancier text lists in some places (ex "Mint, Cerulean and Moss")
  - Creature attributes like "agile" and "bulky" now show up in more places
- Introduce doorways (with the same texture as the ship exit) and use them in place of certain doors in abandoned facility locations
- Lowered weapon damage of bats
- Tweaked the game view: Backgrounds and game objects have been moved up
- Background textures have been tweaked, indoor backgrounds are now slightly smaller vertically
- Updated format for `backgrounds.json`, which now allows y offsets to be specified
- Renamed `crew_data.json` to `starting_crew.json`
- Switched to a different graphics framework
- The primary text box is now built different (uses a scroll bar for large text instead of expanding vertically)
- "status" command now shows the crew status in a gui window

### Fixed

- Normal location load errors are now handled gracefully,
 showing the error message in-gui and letting the game save and return to the main menu
- Slight improvement of messages for some other load errors
- With executable flag `--disable-autosave`, the save file is no longer removed when reaching the victory/game over frame
- Fix pixel-alignment for odd-number-width object texture layers

### Removed

- Text-only view is no longer included

### Contributors for this release

- kirderf1


## [v0.9.0] - 2024-07-06

### Added

- New data file `character_profiles.json`, used for randomly-selected characters
- New aftik colors and character profiles
- Containers that can be searched for items (types include: tents, cabinets, drawers, crates, chests)
- Two new items that can change character stats
- "Hunting missions" where a character gives a reward for killing specific creatures
 (one such mission has been added to the village location)

### Changed

- Save format version 3.1
- Crew size limit when recruiting has been raised from 2 to 3
- Characters may now push crew members that are in the way for certain actions
- Recruitable characters and crew members in json definitions can now be set to use a random character profile
- Aftik corpses can now be set to use a color from a random character profile
- The "recruitable" symbol type (for location defintions) has now been replaced by a more general "character" symbol type
- Loot tables are no longer hardcoded, and can be defined in `assets/loot_table/`
- Locations now use new loot tables `resource` and `tool`,
 which together cover the same items as the original `regular` loot table
- All locations have been updated to use some degree of visually-connected paths
- The tent in the eyesaur forest is now a searchable container
- Containers have been added in a lot of places in most locations
- "status" command now also shows ship fuel status and number of food rations in the ship
- Slight changes to how dialogue goes with recruitable characters

### Fixed

- Save file is now removed when reaching the victory/game over frame
- After victory/game over, the game returns to the main menu
- "Load Game" button is no longer shown when the save file is absent
- The player may no longer command a character to talk to themselves
- Made certain action failures more visible, updated some action failure messages
- Tweaked rendered offset between objects at the same coordinate
- Improved how adjacent objects are rendered over each other
- Space out text from "status" command with empty lines between characters
- Crew members told to wait will now stop waiting when they leave the ship from arriving at a new location
- Tooltips now go in front of the input field
- Tooltips are now clamped to not go below the bottom of the game window
- Change the type of click to advance to the next frame,
 fixes accidentally advancing a frame when clicking something else that is in the same area as the text box

### Removed

- Default symbols for most items have been removed from `symbols.json`
 (these symbols are instead now defined at the specific areas where they are used)
- The `regular` loot table has been removed in favor of using `resource` and `tool`
- Crowbars and bats have been removed from the `valuable` loot table

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
- "check \<item\>" command, which gives information on an item
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
- New commands "talk to \<target\>", "tell \<target\> to wait", "tell \<target\> to follow"
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
