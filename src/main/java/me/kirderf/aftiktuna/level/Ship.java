package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;

public final class Ship {
	private final Room room = new Room(4);
	private final Position entrancePos = room.getPosAt(3);
	
	public void createEntrance(Position destination) {
		Location.createDoors(ObjectTypes.SHIP_ENTRANCE, destination, ObjectTypes.SHIP_EXIT, entrancePos, DoorProperty.EMPTY);
	}
	
	public Room getRoom() {
		return room;
	}
}
