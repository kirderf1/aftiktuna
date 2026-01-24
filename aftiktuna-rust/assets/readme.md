
There are different kinds of data filees in here.

## `assets/starting_crew.json`

Contains crew data used when starting a new game.

## `assets/species_data.json`

Defines certain properties of species. Note that this file cannot be used to add new species.

## `assets/aftik_colors.json`

Contains a list of aftik color definitions.

## `assets/selectable_aftik_color_names.json`

Contains a list of aftik colors that may be used for randomly-generated aftiks, together with a list of relevant names for each color.

## `assets/location/*.json`

Each file is a location template, which are used to generate the ingame location when landing.

## `assets/symbols.json`

A global set of object symbols shared between all location template files.

## `assets/area_furnish/*.json`

Each file represents a pool of area furnish templates. The pools can be used by location templates to randomly furnish an area.

## `assets/locations.json`

Determines the locations used by the game, and defines some location metadata.

## `assets/location/crew_ship.json`

Special location that is loaded in as the crew ship when starting a new game.

## `assets/item_types.json`

Contains a list of item types and properties. Note that some items have hardcoded functionality associated with their id.

## `assets/loot_table/*.json`

Each file is a loot table used by game systems to randomly select an item.

## `assets/texture/background/backgrounds.json`

Contains a list of background rendering data.

## `assets/texture/object/*.json`

Each file contains rendering data for a type of rendered object.
