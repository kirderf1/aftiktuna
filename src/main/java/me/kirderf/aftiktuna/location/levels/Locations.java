package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.GameInstance;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.DoorLockedProperty;
import me.kirderf.aftiktuna.object.door.DoorSealedProperty;
import me.kirderf.aftiktuna.object.door.DoorStuckProperty;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Supplier;

public final class Locations {
	private static final List<Supplier<Location>> levels = List.of(
			Locations::abandonedFacility, Locations::abandonedFacility2, Locations::goblinForest, Locations::eyesaurForest);
	
	private final List<Supplier<Location>> unusedLevels = new ArrayList<>(levels);
	
	public static void checkLocations() {
		for (Supplier<Location> level : levels) {
			level.get();
		}
	}
	
	public Location getRandomLocation() {
		int i = GameInstance.RANDOM.nextInt(unusedLevels.size());
		return unusedLevels.remove(i).get();
	}
	
	private static Location abandonedFacility() {
		LocationBuilder builder = new LocationBuilder();
		Area field = builder.newArea("Field in front of a building", 6);
		Area rightField = builder.newArea("Field", 5);
		Area entrance = builder.newArea("Entrance hall", 7);
		Area corridor1 = builder.newArea("Corridor", 5);
		Area corridor2 = builder.newArea("Corridor", 5);
		Area room1 = builder.newArea("Room", 4);
		Area room2 = builder.newArea("Room", 4);
		Area room3 = builder.newArea("Room", 4);
		Area sealedRoom = builder.newArea("Room", 4);
		
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
		sealedRoom.addItem(ObjectTypes.SWORD, 3);
		sealedRoom.addCreature(ObjectTypes.AZURECLOPS, 1);
		
		return builder.build(field.getPosAt(0));
	}
	
	private static Location abandonedFacility2() {
		LocationBuilder builder = new LocationBuilder();
		Area field = builder.newArea("Field in front of a building", 6);
		Area leftField = builder.newArea("Field next to a building", 5);
		Area entrance = builder.newArea("Entrance hall", 6);
		Area corridor = builder.newArea("Corridor", 6);
		Area sideEntrance = builder.newArea("Corridor", 5);
		Area room1 = builder.newArea("Room", 4);
		Area room2 = builder.newArea("Room", 4);
		Area room3 = builder.newArea("Room", 4);
		Area storage = builder.newArea("Storage Room", 3);
		
		builder.markDoors(field.getPosAt(3), entrance.getPosAt(3));
		builder.createDoors(ObjectTypes.LEFT_PATH, field.getPosAt(0), ObjectTypes.PATH, leftField.getPosAt(4));
		builder.markDoors(leftField.getPosAt(0), sideEntrance.getPosAt(0), new DoorStuckProperty());
		
		builder.markDoors(entrance.getPosAt(0), corridor.getPosAt(0));
		builder.markDoors(entrance.getPosAt(5), room1.getPosAt(0));
		room1.addItem(ObjectTypes.BLOWTORCH, 3);
		room1.addItem(ObjectTypes.SWORD, 3);
		entrance.addCreature(ObjectTypes.EYESAUR, 1);
		room1.addCreature(ObjectTypes.AZURECLOPS, 2);
		
		builder.markDoors(corridor.getPosAt(3), sideEntrance.getPosAt(2));
		builder.markDoors(corridor.getPosAt(5), room2.getPosAt(0));
		corridor.addItem(ObjectTypes.KNIFE, 2);
		room2.addItem(ObjectTypes.CROWBAR, 3);
		corridor.addCreature(ObjectTypes.EYESAUR, 1);
		
		builder.markDoors(sideEntrance.getPosAt(4), room3.getPosAt(0));
		builder.markDoors(room3.getPosAt(3), storage.getPosAt(0), new DoorSealedProperty());
		room3.addItem(ObjectTypes.FUEL_CAN, 2);
		storage.addItem(ObjectTypes.FUEL_CAN, 2);
		room3.addCreature(ObjectTypes.GOBLIN, 1);
		
		return builder.build(field.getPosAt(5));
	}
	
	private static Location goblinForest() {
		LocationBuilder builder = new LocationBuilder();
		Area field = builder.newArea("Field in front of a forest", 7);
		Area entrance = builder.newArea("Forest entrance", 5);
		Area leftPath = builder.newArea("Forest path leading to a shack", 6);
		Area leftPath2 = builder.newArea("Forest path", 5);
		Area rightPath = builder.newArea("Forest path", 6);
		Area rightPath2 = builder.newArea("Forest path", 6);
		Area shack = builder.newArea("Shack", 4);
		
		builder.markPath(field.getPosAt(4), entrance.getPosAt(2));
		builder.markPath(entrance.getPosAt(0), leftPath.getPosAt(4));
		builder.markPath(entrance.getPosAt(4), rightPath.getPosAt(2));
		
		builder.markPath(rightPath.getPosAt(0), rightPath2.getPosAt(5));
		builder.markPath(rightPath2.getPosAt(0), leftPath2.getPosAt(4));
		rightPath.addItem(ObjectTypes.CROWBAR, 5);
		rightPath2.addItem(ObjectTypes.KNIFE, 1);
		rightPath.addCreature(ObjectTypes.GOBLIN, 4);
		rightPath2.addCreature(ObjectTypes.GOBLIN, 2);
		rightPath2.addCreature(ObjectTypes.GOBLIN, 3);
		
		builder.markPath(leftPath.getPosAt(2), leftPath2.getPosAt(0));
		builder.markDoors(leftPath.getPosAt(0), shack.getPosAt(3), new DoorStuckProperty());
		shack.addItem(ObjectTypes.FUEL_CAN, 0);
		shack.addCreature(ObjectTypes.EYESAUR, 1);
		leftPath2.addCreature(ObjectTypes.GOBLIN, 3);
		
		return builder.build(field.getPosAt(1));
	}
	
	private static Location eyesaurForest() {
		LocationBuilder builder = new LocationBuilder();
		Area field = builder.newArea("Field in front of a forest", 8);
		Area leftEntrance = builder.newArea("Forest entrance", 6);
		Area leftPath = builder.newArea("Forest path", 6);
		Area leftPath2 = builder.newArea("Forest path", 4);
		Area rightEntrance = builder.newArea("Forest entrance", 6);
		Area rightPath = builder.newArea("Forest path", 5);
		Area rightPath2 = builder.newArea("Forest path", 5);
		Area midPath = builder.newArea("Forest path", 6);
		Area midPath2 = builder.newArea("Forest path", 7);
		
		builder.markPath(field.getPosAt(1), leftEntrance.getPosAt(3));
		builder.markPath(leftEntrance.getPosAt(0), leftPath.getPosAt(2));
		builder.markPath(leftPath.getPosAt(0), leftPath2.getPosAt(0));
		leftPath.addItem(ObjectTypes.FUEL_CAN, 4);
		leftEntrance.addCreature(ObjectTypes.EYESAUR, 1);
		leftPath2.addCreature(ObjectTypes.EYESAUR, 1);
		
		builder.markPath(leftEntrance.getPosAt(5), midPath.getPosAt(0));
		builder.markPath(rightEntrance.getPosAt(0), midPath.getPosAt(5));
		builder.markPath(midPath.getPosAt(2), midPath2.getPosAt(3));
		builder.markPath(leftPath2.getPosAt(3), midPath2.getPosAt(0));
		builder.markPath(rightPath.getPosAt(0), midPath2.getPosAt(5));
		midPath2.addItem(ObjectTypes.KNIFE, 2);
		midPath.addCreature(ObjectTypes.EYESAUR, 3);
		
		builder.markPath(field.getPosAt(6), rightEntrance.getPosAt(2));
		builder.markPath(rightEntrance.getPosAt(4), rightPath.getPosAt(4));
		builder.markPath(rightPath.getPosAt(2), rightPath2.getPosAt(0));
		rightEntrance.addItem(ObjectTypes.CROWBAR, 3);
		rightPath2.addItem(ObjectTypes.BLOWTORCH, 4);
		rightPath2.addItem(ObjectTypes.FUEL_CAN, 4);
		rightPath.addCreature(ObjectTypes.EYESAUR, 1);
		rightPath2.addCreature(ObjectTypes.EYESAUR, 2);
		rightPath2.addCreature(ObjectTypes.EYESAUR, 3);
		
		return builder.build(field.getPosAt(3));
	}
}
