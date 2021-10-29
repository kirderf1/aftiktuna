package me.kirderf.aftiktuna.level;

public final class Location {
	private final Room room;
	private final int entryPoint;
	
	public Location(Room room, int entryPoint) {
		this.room = room;
		this.entryPoint = entryPoint;
	}
	
	public void addAtEntry(GameObject object) {
		room.addObject(object, entryPoint);
	}
}