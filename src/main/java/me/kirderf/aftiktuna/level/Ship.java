package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;

import java.util.stream.Collectors;

public final class Ship {
	private final Room room = new Room(4);
	private final Position entrancePos = room.getPosAt(3);
	
	public void separateFromLocation() {
		room.objectStream().filter(ObjectTypes.SHIP_EXIT::matching).collect(Collectors.toList()).forEach(GameObject::remove);
	}
	
	public void createEntrance(Position destination) {
		Location.createDoors(ObjectTypes.SHIP_ENTRANCE, destination, ObjectTypes.SHIP_EXIT, entrancePos, DoorProperty.EMPTY);
	}
	
	public Room getRoom() {
		return room;
	}
}
