package me.kirderf.aftiktuna;

public final class Location {
	final Room room;
	final int entryPoint;
	final GameObject fuelCan;
	
	public Location(Room room, int entryPoint, GameObject fuelCan) {
		this.room = room;
		this.entryPoint = entryPoint;
		this.fuelCan = fuelCan;
	}
}