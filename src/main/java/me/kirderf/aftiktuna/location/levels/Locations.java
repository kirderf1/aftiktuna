package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.Room;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.object.door.DoorSealedProperty;
import me.kirderf.aftiktuna.object.door.DoorStuckProperty;
import me.kirderf.aftiktuna.object.entity.Creature;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;

public final class Locations {
	private static final List<Supplier<Location>> levels = List.of(Locations::createCrowbarLocation, Locations::createBlowtorchLocation, Locations::createKeycardLocation);
	
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
		rightRoom.addObject(new Creature(ObjectTypes.AZURECLOPS, true), 2);
		
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
		secondRoom.addObject(new Creature(ObjectTypes.EYESAUR, true), 1);
		
		return builder.build(firstRoom.getPosAt(0));
	}
	
	private static Location createKeycardLocation() {
		LocationBuilder builder = new LocationBuilder();
		Room firstRoom = builder.newRoom(6);
		Room leftRoom = builder.newRoom(3);
		Room rightRoom = builder.newRoom(4);
		builder.markDoors(firstRoom.getPosAt(2), leftRoom.getPosAt(2), new DoorLockedProperty());
		builder.markDoors(firstRoom.getPosAt(5), rightRoom.getPosAt(1));
		leftRoom.addItem(ObjectTypes.FUEL_CAN, 0);
		leftRoom.addItem(ObjectTypes.FUEL_CAN, 1);
		rightRoom.addItem(ObjectTypes.KEYCARD, 0);
		rightRoom.addObject(new Creature(ObjectTypes.GOBLIN, true), 2);
		rightRoom.addObject(new Creature(ObjectTypes.GOBLIN, true), 3);
		
		return builder.build(firstRoom.getPosAt(0));
	}
}
