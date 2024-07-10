Hello! Here are some basic instructions to fill in spaces that the game can't do itself in its current form.

The goal of the game is to travel from planet to planet, collecting fuel cans to refuel the ship.

The game has a graphical view and a text-only view. You can press tab to switch between them.

You interact with the game by giving commands to a character in the crew, making them perform various actions.
Currently available commands are:
- take <item>
- take all
- search <container>
- give <character> <item>
- check <item>
- wield <item>
- use <item>
- enter <path/door>
- go to ship
- force <door>
- attack <creature>
- attack
- wait
- rest
- refuel ship
- launch ship
- status
- control <character>
- trade
- recruit aftik
- talk to <character>
- tell <character> to wait
- tell <character> to wait at ship
- tell <character> to follow
- open fortuna chest

When trading at a store, there are the following commands:
- buy <item>
- buy <number> <items>
- sell <item>
- sell <number> <items>
- sell all <items>
- exit
- status

You can at any time give the command "exit game" to quit and return to the main menu.

Notes:
- "rest" doesn't recover health, instead it waits until stamina (a stat that helps with dodging attacks) has recovered for the crew.
- Health is partially recovered when moving with the ship from one location to another, as long as the crew has food rations to consume for it. This happens automatically, and it doesn't matter which character is holding the food rations.
- Use the mouse to drag the camera view in larger areas indicated by white arrows.
- You can also use the mouse to see the name of objects in view.
- You can click game objects to get a list of command suggestions. Note that this does not cover all possible commands that you might want to do.
- The game saves automatically when closed. It's possible to disable autosaving by passing in "--disable-autosave" as a flag when running the executable.
