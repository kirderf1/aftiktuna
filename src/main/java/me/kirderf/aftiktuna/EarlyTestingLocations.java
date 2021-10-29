package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Room;

@SuppressWarnings("unused")
public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		Room room = new Room(5);
		room.addObject(new FuelCan(), 4);
		return new Location(room.getPosAt(1));
	}
	
	public static Location createLocation2() {
		Room room = new Room(4);
		room.addObject(new FuelCan(), 0);
		room.addObject(new FuelCan(), 3);
		return new Location(room.getPosAt(1));
	}
	
	public static Location createLocation3() {
		Room room = new Room(3);
		room.addObject(new FuelCan(), 2);
		room.addObject(new FuelCan(), 2);
		return new Location(room.getPosAt(0));
	}
}
