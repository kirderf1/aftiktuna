package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.Door;
import me.kirderf.aftiktuna.level.object.FuelCan;

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
	
	public static Location createDoorLocation1() {
		Room firstRoom = new Room(3);
		Room secondRoom = new Room(3);
		firstRoom.addObject(new Door(secondRoom.getPosAt(0)), 2);
		secondRoom.addObject(new Door(firstRoom.getPosAt(2)), 0);
		secondRoom.addObject(new FuelCan(), 2);
		return new Location(firstRoom.getPosAt(0));
	}
}
