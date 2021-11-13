package me.kirderf.aftiktuna.level.object.door;

import me.kirderf.aftiktuna.level.Room;

public final record DoorPair(Door targetDoor, Room destination) {}