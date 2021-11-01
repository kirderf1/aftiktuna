package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.door.DoorProperty;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;

public final class LocationBuilder {
	private final List<Room> rooms = new ArrayList<>();
	
	public Room newRoom(int size) {
		Room room = new Room(size);
		rooms.add(room);
		return room;
	}
	
	public void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2) {
		createDoors(type1, pos1, type2, pos2, DoorProperty.EMPTY);
	}
	
	public void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2, DoorProperty property) {
		verifyPosition(pos1);
		verifyPosition(pos2);
		AtomicReference<DoorProperty> atomic = new AtomicReference<>(property);
		pos1.room().addObject(new Door(type1, pos2, atomic), pos1.coord());
		pos2.room().addObject(new Door(type2, pos1, atomic), pos2.coord());
	}
	
	public Location build(Position entryPos) {
		verifyPosition(entryPos);
		return new Location(rooms, entryPos);
	}
	
	private void verifyPosition(Position pos) {
		if (!rooms.contains(pos.room()))
			throw new IllegalArgumentException("Illegal position: room is not of this location!");
	}
}
