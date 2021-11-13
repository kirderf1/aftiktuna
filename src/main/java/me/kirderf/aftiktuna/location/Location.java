package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;

import java.util.List;
import java.util.concurrent.atomic.AtomicReference;

public final class Location {
	private final List<Room> rooms;
	private final Position entryPos;
	
	Location(List<Room> rooms, Position entryPos) {
		this.rooms = List.copyOf(rooms);
		this.entryPos = entryPos;
	}
	
	public List<Room> getRooms() {
		return rooms;
	}
	
	public Position getEntryPos() {
		return entryPos;
	}
	
	public void addAtEntry(GameObject object) {
		entryPos.room().addObject(object, entryPos);
	}
	
	static void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> atomic = new AtomicReference<>(property);
		pos1.room().addObject(new Door(type1, pos2, atomic), pos1.coord());
		pos2.room().addObject(new Door(type2, pos1, atomic), pos2.coord());
	}
}