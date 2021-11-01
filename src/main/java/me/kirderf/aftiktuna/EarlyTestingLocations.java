package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.LocationBuilder;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.Creature;
import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.level.object.door.DoorSealedProperty;
import me.kirderf.aftiktuna.level.object.door.DoorStuckProperty;

@SuppressWarnings("unused")
public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		LocationBuilder builder = new LocationBuilder();
		Room room = builder.newRoom(5);
		room.addItem(ObjectType.FUEL_CAN, 4);
		return builder.build(room.getPosAt(1));
	}
	
	public static Location createLocation2() {
		LocationBuilder builder = new LocationBuilder();
		Room room = builder.newRoom(4);
		room.addItem(ObjectType.FUEL_CAN, 0);
		room.addItem(ObjectType.FUEL_CAN, 3);
		return builder.build(room.getPosAt(1));
	}
	
	public static Location createLocation3() {
		LocationBuilder builder = new LocationBuilder();
		Room room = builder.newRoom(3);
		room.addItem(ObjectType.FUEL_CAN, 2);
		room.addItem(ObjectType.FUEL_CAN, 2);
		return builder.build(room.getPosAt(0));
	}
	
	public static Location createDoorLocation1() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(3);
		Room secondRoom = builder.newRoom(3);
		builder.createDoors(ObjectType.DOOR, firstRoom.getPosAt(2), ObjectType.DOOR, secondRoom.getPosAt(0));
		secondRoom.addItem(ObjectType.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createDoorLocation2() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(3);
		Room leftRoom = builder.newRoom(3);
		Room rightRoom = builder.newRoom(3);
		builder.createDoors(ObjectType.LEFT_DOOR, firstRoom.getPosAt(1), ObjectType.LEFT_DOOR, leftRoom.getPosAt(0));
		builder.createDoors(ObjectType.RIGHT_DOOR, firstRoom.getPosAt(2), ObjectType.RIGHT_DOOR, rightRoom.getPosAt(1));
		builder.createDoors(ObjectType.RIGHT_DOOR, leftRoom.getPosAt(2), ObjectType.LEFT_DOOR, rightRoom.getPosAt(0));
		rightRoom.addItem(ObjectType.FUEL_CAN, 2);
		rightRoom.addItem(ObjectType.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createToolsLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(4);
		Room secondRoom = builder.newRoom(2);
		Room thirdRoom = builder.newRoom(3);
		Room sideRoom = builder.newRoom(3);
		firstRoom.addItem(ObjectType.CROWBAR, 2);
		firstRoom.addItem(ObjectType.KEYCARD, 2);
		builder.createDoors(ObjectType.LEFT_DOOR, firstRoom.getPosAt(1), ObjectType.RIGHT_DOOR, secondRoom.getPosAt(1), new DoorStuckProperty());
		builder.createDoors(ObjectType.RIGHT_DOOR, firstRoom.getPosAt(3), ObjectType.DOOR, sideRoom.getPosAt(0), new DoorSealedProperty());
		secondRoom.addItem(ObjectType.BLOWTORCH, 0);
		builder.createDoors(ObjectType.LEFT_DOOR, secondRoom.getPosAt(0), ObjectType.DOOR, thirdRoom.getPosAt(0), new DoorLockedProperty());
		thirdRoom.addItem(ObjectType.FUEL_CAN, 2);
		sideRoom.addItem(ObjectType.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createBlockingLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(6);
		Room secondRoom = builder.newRoom(3);
		builder.createDoors(ObjectType.LEFT_DOOR, firstRoom.getPosAt(1), ObjectType.LEFT_DOOR, secondRoom.getPosAt(0));
		builder.createDoors(ObjectType.RIGHT_DOOR, firstRoom.getPosAt(3), ObjectType.RIGHT_DOOR, secondRoom.getPosAt(2));
		firstRoom.addObject(new Creature(false), 2);
		firstRoom.addItem(ObjectType.CROWBAR, 0);
		firstRoom.addItem(ObjectType.FUEL_CAN, 2);
		firstRoom.addItem(ObjectType.FUEL_CAN, 5);
		firstRoom.addItem(ObjectType.KEYCARD, 5);
		secondRoom.addItem(ObjectType.KNIFE, 1);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createDeathLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room room = builder.newRoom(5);
		room.addObject(new Creature(true), 2);
		room.addObject(new Creature(true), 2);
		room.addObject(new Creature(true), 3);
		room.addItem(ObjectType.FUEL_CAN, 4);
		return builder.build(room.getPosAt(0));
	}
}