package me.kirderf.aftiktuna.level;

public final class Location {
	private final Position entryPos;
	
	public Location(Position entryPos) {
		this.entryPos = entryPos;
	}
	
	public void addAtEntry(GameObject object) {
		entryPos.room().addObject(object, entryPos);
	}
}