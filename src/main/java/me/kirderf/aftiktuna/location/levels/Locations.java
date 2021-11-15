package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.object.door.DoorSealedProperty;
import me.kirderf.aftiktuna.object.door.DoorStuckProperty;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;

public final class Locations {
	private static final List<Supplier<Location>> levels = List.of(Locations::createCrowbarLocation, Locations::createBlowtorchLocation,
			Locations::abandonedFacility, Locations::abandonedFacility2);
	
	private final List<Supplier<Location>> unusedLevels = new ArrayList<>(levels);
	
	public Location getRandomLocation() {
		int i = GameInstance.RANDOM.nextInt(unusedLevels.size());
		return unusedLevels.remove(i).get();
	}
	
	private static Location createCrowbarLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(7);
		Room leftRoom = builder.newRoom(3);
		Room rightRoom = builder.newRoom(4);
		builder.markDoors(firstRoom.getPosAt(1), leftRoom.getPosAt(2));
		builder.markDoors(firstRoom.getPosAt(5), rightRoom.getPosAt(0), new DoorStuckProperty());
		leftRoom.addItem(ObjectTypes.CROWBAR, 1);
		rightRoom.addItem(ObjectTypes.FUEL_CAN, 3);
		rightRoom.addCreature(ObjectTypes.AZURECLOPS, 2);
		
		return builder.build(firstRoom.getPosAt(3));
	}
	
	private static Location createBlowtorchLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(3);
		Room secondRoom = builder.newRoom(6);
		Room thirdRoom = builder.newRoom(4);
		builder.markDoors(firstRoom.getPosAt(2), secondRoom.getPosAt(3));
		builder.markDoors(secondRoom.getPosAt(4), thirdRoom.getPosAt(1), new DoorSealedProperty());
		secondRoom.addItem(ObjectTypes.BLOWTORCH, 0);
		secondRoom.addItem(ObjectTypes.FUEL_CAN, 5);
		thirdRoom.addItem(ObjectTypes.KNIFE, 3);
		secondRoom.addCreature(ObjectTypes.EYESAUR, 1);
		
		return builder.build(firstRoom.getPosAt(0));
	}
	
	private static Location abandonedFacility() {
		LocationBuilder builder = new LocationBuilder();
		Room field = builder.newRoom(6);
		Room rightField = builder.newRoom(5);
		Room entrance = builder.newRoom(7);
		Room corridor1 = builder.newRoom(5);
		Room corridor2 = builder.newRoom(5);
		Room room1 = builder.newRoom(4);
		Room room2 = builder.newRoom(4);
		Room room3 = builder.newRoom(4);
		Room sealedRoom = builder.newRoom(4);
		
		builder.markDoors(field.getPosAt(2), entrance.getPosAt(5), new DoorLockedProperty());
		builder.createDoors(ObjectTypes.RIGHT_PATH, field.getPosAt(5), ObjectTypes.PATH, rightField.getPosAt(0));
		rightField.addItem(ObjectTypes.KEYCARD, 3);
		
		builder.markDoors(entrance.getPosAt(1), sealedRoom.getPosAt(2), new DoorSealedProperty());
		builder.markDoors(entrance.getPosAt(3), corridor1.getPosAt(0));
		builder.markDoors(corridor1.getPosAt(2), room1.getPosAt(3));
		builder.markDoors(corridor1.getPosAt(4), corridor2.getPosAt(0));
		builder.markDoors(corridor2.getPosAt(2), room2.getPosAt(3), new DoorStuckProperty());
		builder.markDoors(corridor2.getPosAt(4), room3.getPosAt(0));
		room1.addItem(ObjectTypes.CROWBAR, 1);
		corridor1.addCreature(ObjectTypes.EYESAUR, 3);
		room2.addItem(ObjectTypes.BLOWTORCH, 0);
		room3.addItem(ObjectTypes.FUEL_CAN, 3);
		room3.addCreature(ObjectTypes.EYESAUR, 1);
		room3.addCreature(ObjectTypes.EYESAUR, 2);
		sealedRoom.addItem(ObjectTypes.FUEL_CAN, 0);
		sealedRoom.addItem(ObjectTypes.KNIFE, 3);
		sealedRoom.addCreature(ObjectTypes.AZURECLOPS, 1);
		
		return builder.build(field.getPosAt(0));
	}
	
	private static Location abandonedFacility2() {
		LocationBuilder builder = new LocationBuilder();
		Room field = builder.newRoom(6);
		Room leftField = builder.newRoom(5);
		Room entrance = builder.newRoom(6);
		Room corridor = builder.newRoom(6);
		Room sideEntrance = builder.newRoom(5);
		Room room1 = builder.newRoom(4);
		Room room2 = builder.newRoom(4);
		Room room3 = builder.newRoom(4);
		Room storage = builder.newRoom(3);
		
		builder.markDoors(field.getPosAt(3), entrance.getPosAt(3));
		builder.createDoors(ObjectTypes.LEFT_PATH, field.getPosAt(0), ObjectTypes.PATH, leftField.getPosAt(4));
		builder.markDoors(leftField.getPosAt(0), sideEntrance.getPosAt(0), new DoorStuckProperty());
		
		builder.markDoors(entrance.getPosAt(0), corridor.getPosAt(0));
		builder.markDoors(entrance.getPosAt(5), room1.getPosAt(0));
		room1.addItem(ObjectTypes.BLOWTORCH, 3);
		entrance.addCreature(ObjectTypes.EYESAUR, 1);
		room1.addCreature(ObjectTypes.AZURECLOPS, 2);
		
		builder.markDoors(corridor.getPosAt(3), sideEntrance.getPosAt(3));
		builder.markDoors(corridor.getPosAt(5), room2.getPosAt(0));
		corridor.addItem(ObjectTypes.KNIFE, 2);
		room2.addItem(ObjectTypes.CROWBAR, 3);
		corridor.addCreature(ObjectTypes.EYESAUR, 1);
		
		builder.markDoors(sideEntrance.getPosAt(5), room3.getPosAt(0));
		builder.markDoors(room3.getPosAt(3), storage.getPosAt(0), new DoorSealedProperty());
		room3.addItem(ObjectTypes.FUEL_CAN, 2);
		storage.addItem(ObjectTypes.FUEL_CAN, 2);
		room3.addCreature(ObjectTypes.GOBLIN, 1);
		
		return builder.build(field.getPosAt(5));
	}
}
