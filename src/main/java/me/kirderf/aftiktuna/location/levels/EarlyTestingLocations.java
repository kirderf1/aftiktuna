package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

@SuppressWarnings("unused")
public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		LocationBuilder builder = new LocationBuilder();
		Area room = builder.newTestRoom(5);
		room.addItem(ObjectTypes.FUEL_CAN, 4);
		return builder.build(room.getPosAt(1));
	}
	
	public static Location createLocation2() {
		LocationBuilder builder = new LocationBuilder();
		Area room = builder.newTestRoom(4);
		room.addItem(ObjectTypes.FUEL_CAN, 0);
		room.addItem(ObjectTypes.FUEL_CAN, 3);
		return builder.build(room.getPosAt(1));
	}
	
	public static Location createLocation3() {
		LocationBuilder builder = new LocationBuilder();
		Area room = builder.newTestRoom(3);
		room.addItem(ObjectTypes.FUEL_CAN, 2);
		room.addItem(ObjectTypes.FUEL_CAN, 2);
		return builder.build(room.getPosAt(0));
	}
	
	public static Location createDoorLocation1() {
		LocationBuilder builder = new LocationBuilder();
		Area firstRoom = builder.newTestRoom(3);
		Area secondRoom = builder.newTestRoom(3);
		builder.markDoors(firstRoom.getPosAt(2), secondRoom.getPosAt(0));
		secondRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createDoorLocation2() {
		LocationBuilder builder = new LocationBuilder();
		Area firstRoom = builder.newTestRoom(3);
		Area leftRoom = builder.newTestRoom(3);
		Area rightRoom = builder.newTestRoom(3);
		builder.markDoors(firstRoom.getPosAt(1), leftRoom.getPosAt(0));
		builder.markDoors(firstRoom.getPosAt(2), rightRoom.getPosAt(1));
		builder.markDoors(leftRoom.getPosAt(2), rightRoom.getPosAt(0));
		rightRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		rightRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createToolsLocation() {
		LocationBuilder builder = new LocationBuilder();
		Area firstRoom = builder.newTestRoom(4);
		Area secondRoom = builder.newTestRoom(2);
		Area thirdRoom = builder.newTestRoom(3);
		Area sideRoom = builder.newTestRoom(3);
		firstRoom.addItem(ObjectTypes.CROWBAR, 2);
		firstRoom.addItem(ObjectTypes.KEYCARD, 2);
		builder.markDoors(firstRoom.getPosAt(1), secondRoom.getPosAt(1), DoorProperty.STUCK);
		builder.markDoors(firstRoom.getPosAt(3), sideRoom.getPosAt(0), DoorProperty.SEALED);
		secondRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		builder.markDoors(secondRoom.getPosAt(0), thirdRoom.getPosAt(0), DoorLockedProperty.INSTANCE);
		thirdRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		sideRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createBlockingLocation() {
		LocationBuilder builder = new LocationBuilder();
		Area firstRoom = builder.newTestRoom(6);
		Area secondRoom = builder.newTestRoom(3);
		builder.markDoors(firstRoom.getPosAt(1), secondRoom.getPosAt(0));
		builder.markDoors(firstRoom.getPosAt(3), secondRoom.getPosAt(2));
		firstRoom.addObject(new Creature(ObjectTypes.EYESAUR, false), 2);
		firstRoom.addItem(ObjectTypes.CROWBAR, 0);
		firstRoom.addItem(ObjectTypes.FUEL_CAN, 2);
		firstRoom.addItem(ObjectTypes.FUEL_CAN, 5);
		firstRoom.addItem(ObjectTypes.KEYCARD, 5);
		secondRoom.addItem(ObjectTypes.KNIFE, 1);
		return builder.build(firstRoom.getPosAt(0));
	}
	
	public static Location createDeathLocation() {
		LocationBuilder builder = new LocationBuilder();
		Area room = builder.newTestRoom(5);
		room.addCreature(ObjectTypes.AZURECLOPS, 2);
		room.addCreature(ObjectTypes.AZURECLOPS, 2);
		room.addCreature(ObjectTypes.AZURECLOPS, 3);
		room.addItem(ObjectTypes.FUEL_CAN, 4);
		return builder.build(room.getPosAt(0));
	}
}