package me.kirderf.aftiktuna;

public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		Room room = new Room(5);
		GameObject fuelCan;
		room.addObject(fuelCan = new GameObject('f', "Fuel can"), 4);
		return new Location(room, 1, fuelCan);
	}
}
