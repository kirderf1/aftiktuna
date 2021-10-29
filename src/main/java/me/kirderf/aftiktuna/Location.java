package me.kirderf.aftiktuna;

public final class Location {
	final Room room;
	final int entryPoint;
	
	public Location(Room room, int entryPoint) {
		this.room = room;
		this.entryPoint = entryPoint;
	}
}