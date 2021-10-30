package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.Position;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.*;

import java.util.concurrent.atomic.AtomicReference;

@SuppressWarnings("unused")
public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		Room room = new Room(5);
		room.addObject(new Item(ObjectType.FUEL_CAN), 4);
		return new Location(room.getPosAt(1));
	}
	
	public static Location createLocation2() {
		Room room = new Room(4);
		room.addObject(new Item(ObjectType.FUEL_CAN), 0);
		room.addObject(new Item(ObjectType.FUEL_CAN), 3);
		return new Location(room.getPosAt(1));
	}
	
	public static Location createLocation3() {
		Room room = new Room(3);
		room.addObject(new Item(ObjectType.FUEL_CAN), 2);
		room.addObject(new Item(ObjectType.FUEL_CAN), 2);
		return new Location(room.getPosAt(0));
	}
	
	public static Location createDoorLocation1() {
		Room firstRoom = new Room(3);
		Room secondRoom = new Room(3);
		createDoors(ObjectType.DOOR, firstRoom.getPosAt(2), ObjectType.DOOR, secondRoom.getPosAt(0));
		secondRoom.addObject(new Item(ObjectType.FUEL_CAN), 2);
		return new Location(firstRoom.getPosAt(0));
	}
	
	public static Location createDoorLocation2() {
		Room firstRoom = new Room(3);
		Room leftRoom = new Room(3);
		Room rightRoom = new Room(3);
		createDoors(ObjectType.LEFT_DOOR, firstRoom.getPosAt(1), ObjectType.LEFT_DOOR, leftRoom.getPosAt(0));
		createDoors(ObjectType.RIGHT_DOOR, firstRoom.getPosAt(2), ObjectType.RIGHT_DOOR, rightRoom.getPosAt(1));
		createDoors(ObjectType.RIGHT_DOOR, leftRoom.getPosAt(2), ObjectType.LEFT_DOOR, rightRoom.getPosAt(0));
		rightRoom.addObject(new Item(ObjectType.FUEL_CAN), 2);
		rightRoom.addObject(new Item(ObjectType.FUEL_CAN), 2);
		return new Location(firstRoom.getPosAt(0));
	}
	
	public static Location createDoorLocation3() {
		Room firstRoom = new Room(3);
		Room secondRoom = new Room(2);
		Room thirdRoom = new Room(3);
		firstRoom.addObject(new Item(ObjectType.CROWBAR), 2);
		firstRoom.addObject(new Item(ObjectType.KEYCARD), 2);
		createDoors(ObjectType.DOOR, firstRoom.getPosAt(1), ObjectType.RIGHT_DOOR, secondRoom.getPosAt(1), new DoorStuckProperty());
		secondRoom.addObject(new Item(ObjectType.BLOWTORCH), 0);
		createDoors(ObjectType.LEFT_DOOR, secondRoom.getPosAt(0), ObjectType.DOOR, thirdRoom.getPosAt(0), new DoorLockedProperty());
		thirdRoom.addObject(new Item(ObjectType.FUEL_CAN), 2);
		return new Location(firstRoom.getPosAt(0));
	}
	
	private static void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2) {
		createDoors(type1, pos1, type2, pos2, DoorProperty.EMPTY);
	}
	private static void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> atomic = new AtomicReference<>(property);
		pos1.room().addObject(new Door(type1, pos2, atomic), pos1.coord());
		pos2.room().addObject(new Door(type2, pos1, atomic), pos2.coord());
	}
}
