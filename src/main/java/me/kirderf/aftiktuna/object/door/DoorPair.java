package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.location.Room;

public final record DoorPair(Door targetDoor, Room destination) {}