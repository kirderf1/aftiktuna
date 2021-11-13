package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;

import java.util.*;
import java.util.concurrent.atomic.AtomicReference;

public final class LocationBuilder {
	private final List<Room> rooms = new ArrayList<>();
	private final Map<Room, List<DoorMark>> doorMap = new HashMap<>();
	
	public Room newRoom(int size) {
		Room room = new Room(size);
		rooms.add(room);
		doorMap.put(room, new ArrayList<>());
		return room;
	}
	
	public void markDoors(Position pos1, Position pos2) {
		markDoors(pos1, pos2, DoorProperty.EMPTY);
	}
	
	public void markDoors(Position pos1, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> reference = new AtomicReference<>(property);
		doorMap.get(pos1.room()).add(new DoorMark(pos1.coord(), pos2, reference));
		doorMap.get(pos2.room()).add(new DoorMark(pos2.coord(), pos1, reference));
	}
	
	private static record DoorMark(int coord, Position destination, AtomicReference<DoorProperty> property) {}
	
	public void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2) {
		createDoors(type1, pos1, type2, pos2, DoorProperty.EMPTY);
	}
	
	public void createDoors(ObjectType type1, Position pos1, ObjectType type2, Position pos2, DoorProperty property) {
		verifyPosition(pos1);
		verifyPosition(pos2);
		Location.createDoors(type1, pos1, type2, pos2, property);
	}
	
	public Location build(Position entryPos) {
		verifyPosition(entryPos);
		buildDoors();
		return new Location(rooms, entryPos);
	}
	
	private void buildDoors() {
		for (Room room : rooms) {
			List<DoorMark> marks = doorMap.get(room);
			if (marks.size() == 1) {
				DoorMark door = marks.get(0);
				room.addObject(new Door(ObjectTypes.DOOR, door.destination, door.property), door.coord);
			} else if (marks.size() == 2) {
				marks.sort(Comparator.comparingInt(DoorMark::coord));
				DoorMark leftDoor = marks.get(0), rightDoor = marks.get(1);
				room.addObject(new Door(ObjectTypes.LEFT_DOOR, leftDoor.destination, leftDoor.property), leftDoor.coord);
				room.addObject(new Door(ObjectTypes.RIGHT_DOOR, rightDoor.destination, rightDoor.property), rightDoor.coord);
			} else if (marks.size() > 2) {
				throw new IllegalStateException("Marked more than two doors in a room. This is not currently supported.");
			}
		}
	}
	
	private void verifyPosition(Position pos) {
		if (!rooms.contains(pos.room()))
			throw new IllegalArgumentException("Illegal position: room is not of this location!");
	}
}