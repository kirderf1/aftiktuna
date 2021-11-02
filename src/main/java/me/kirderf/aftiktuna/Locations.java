package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.Location;
import me.kirderf.aftiktuna.level.LocationBuilder;
import me.kirderf.aftiktuna.level.Room;
import me.kirderf.aftiktuna.level.object.ObjectTypes;
import me.kirderf.aftiktuna.level.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.level.object.door.DoorSealedProperty;
import me.kirderf.aftiktuna.level.object.door.DoorStuckProperty;
import me.kirderf.aftiktuna.level.object.entity.Creature;

import java.util.List;
import java.util.function.Supplier;

public class Locations {
	private static final List<Supplier<Location>> levels = List.of(Locations::createCrowbarLocation, Locations::createBlowtorchLocation, Locations::createKeycardLocation);
	
	public static Location getRandomLocation() {
		int i = GameInstance.RANDOM.nextInt(levels.size());
		return levels.get(i).get();
	}
	
	private static Location createCrowbarLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(7);
		Room leftRoom = builder.newRoom(3);
		Room rightRoom = builder.newRoom(4);
		builder.createDoors(ObjectTypes.LEFT_DOOR, firstRoom.getPosAt(1), ObjectTypes.DOOR, leftRoom.getPosAt(2));
		builder.createDoors(ObjectTypes.RIGHT_DOOR, firstRoom.getPosAt(5), ObjectTypes.DOOR, rightRoom.getPosAt(0), new DoorStuckProperty());
		leftRoom.addItem(ObjectTypes.CROWBAR, 1);
		rightRoom.addItem(ObjectTypes.FUEL_CAN, 3);
		rightRoom.addObject(new Creature(true), 2);
		
		return builder.build(firstRoom.getPosAt(3));
	}
	
	private static Location createBlowtorchLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(3);
		Room secondRoom = builder.newRoom(6);
		Room thirdRoom = builder.newRoom(4);
		builder.createDoors(ObjectTypes.DOOR, firstRoom.getPosAt(2), ObjectTypes.LEFT_DOOR, secondRoom.getPosAt(3));
		builder.createDoors(ObjectTypes.RIGHT_DOOR, secondRoom.getPosAt(4), ObjectTypes.DOOR, thirdRoom.getPosAt(1), new DoorSealedProperty());
		secondRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		secondRoom.addItem(ObjectTypes.FUEL_CAN, 5);
		thirdRoom.addItem(ObjectTypes.KNIFE, 3);
		secondRoom.addObject(new Creature(true), 1);
		
		return builder.build(firstRoom.getPosAt(0));
	}
	
	private static Location createKeycardLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(6);
		Room leftRoom = builder.newRoom(3);
		Room rightRoom = builder.newRoom(4);
		builder.createDoors(ObjectTypes.LEFT_DOOR, firstRoom.getPosAt(2), ObjectTypes.DOOR, leftRoom.getPosAt(2), new DoorLockedProperty());
		builder.createDoors(ObjectTypes.RIGHT_DOOR, firstRoom.getPosAt(5), ObjectTypes.DOOR, rightRoom.getPosAt(1));
		leftRoom.addItem(ObjectTypes.FUEL_CAN, 0);
		leftRoom.addItem(ObjectTypes.FUEL_CAN, 1);
		rightRoom.addItem(ObjectTypes.KEYCARD, 0);
		rightRoom.addObject(new Creature(true), 2);
		rightRoom.addObject(new Creature(true), 3);
		
		return builder.build(firstRoom.getPosAt(0));
	}
}
