package me.kirderf.aftiktuna.level;

import java.util.List;

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
}