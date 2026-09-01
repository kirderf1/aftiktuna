# Aftiktuna Spec

Here, I try to formalize and keep track of any concepts or design principles that I have settled on. Not everything may match the current state of the game, but are moreso written here to help inform future decisions.

## Intro

The game is an attempt to realize the fictional game of Fortuna from the webcomic of the same name.
A lot of foundational aspects of the game comes from the comic.
However anything on top of that, such as specific locations, scenarios, events, characters and similar, generally do not.
This is after all an attempt to realize the fictional game, rather than to recreate the webcomic.

The game is shown from a 2D sideways view. Gameplay consists of exploration, resource management and decision making.
There are no real-time aspects of the game. Time only progreses on input from the player.
The player gives commands to a crew of characters visiting planets and traversing through space.
Most of the time in the game is spent on ground, exploring and interacting with the environment through an individual character, as well as seeing interactions between characters.
Space travel, or formulating plans or strategies, make up a smaller yet still significant part of the gameplay.

The game has a rougelike flow: The goal is to reach Fortuna, and losing entails starting over with a new crew.
On their journey there, the player visits locations on different planets with the subgoal of gathering resources, primarily fuel.

## Locations

Each location hosts a limited assortment from a wide variety of resources, obstacles, dangers, events and situations. Some are more common than others, and some are unique to specifically designed locations.

A good location centers around **one** main feature / event / scenario, which can be approached or interacted with in multiple different ways.
While a location can be designed to support multiple such things, only one will be at play for each visit to that location.
Any more would give the location too much scope (this is a linear rougelike, not an open world), and any less would produce a plain or repetetive location.

This does not include any additional simpler features or events alongside the "main attraction",
as these simpler features can help fill out a location where the main feature does not.
