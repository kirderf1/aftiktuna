package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;

import java.util.concurrent.atomic.AtomicReference;

public final class Ship {
	private final Room room = new Room(4);
	private final Position entrancePos = room.getPosAt(3);
	
	public void createEntrance(Position destination) {
		AtomicReference<DoorProperty> property = new AtomicReference<>(DoorProperty.EMPTY);
		room.addObject(new Door(ObjectTypes.SHIP_EXIT, destination, property), entrancePos.coord());
		destination.room().addObject(new Door(ObjectTypes.SHIP_ENTRANCE, entrancePos, property), destination.coord());
	}
	
	public Room getRoom() {
		return room;
	}
}
