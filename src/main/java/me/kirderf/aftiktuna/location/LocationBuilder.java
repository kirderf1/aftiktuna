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
	private final Map<Room, List<DoorMark>> pathMap = new HashMap<>();
	
	public Room newRoom(String label, int size) {
		Room room = new Room(label, size);
		rooms.add(room);
		doorMap.put(room, new ArrayList<>());
		pathMap.put(room, new ArrayList<>());
		return room;
	}
	
	public Room newTestRoom(int size) {
		return newRoom("Room", size);
	}
	
	public void markDoors(Position pos1, Position pos2) {
		markDoors(pos1, pos2, DoorProperty.EMPTY);
	}
	
	public void markDoors(Position pos1, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> reference = new AtomicReference<>(property);
		doorMap.get(pos1.room()).add(new DoorMark(pos1.coord(), pos2, reference));
		doorMap.get(pos2.room()).add(new DoorMark(pos2.coord(), pos1, reference));
	}
	
	public void markPath(Position pos1, Position pos2) {
		markPath(pos1, pos2, DoorProperty.EMPTY);
	}
	
	public void markPath(Position pos1, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> reference = new AtomicReference<>(property);
		pathMap.get(pos1.room()).add(new DoorMark(pos1.coord(), pos2, reference));
		pathMap.get(pos2.room()).add(new DoorMark(pos2.coord(), pos1, reference));
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
			marks.sort(Comparator.comparingInt(DoorMark::coord));
			if (marks.size() == 1) {
				addMarkedDoor(room, ObjectTypes.DOOR, marks.get(0));
			} else if (marks.size() == 2) {
				addMarkedDoor(room, ObjectTypes.LEFT_DOOR, marks.get(0));
				addMarkedDoor(room, ObjectTypes.RIGHT_DOOR, marks.get(1));
			} else if (marks.size() == 3) {
				addMarkedDoor(room, ObjectTypes.LEFT_DOOR, marks.get(0));
				addMarkedDoor(room, ObjectTypes.MIDDLE_DOOR, marks.get(1));
				addMarkedDoor(room, ObjectTypes.RIGHT_DOOR, marks.get(2));
			} else if (marks.size() > 3) {
				throw new IllegalStateException("Marked more than three doors in a room. This is not currently supported.");
			}
			List<DoorMark> paths = pathMap.get(room);
			paths.sort(Comparator.comparingInt(DoorMark::coord));
			if (paths.size() == 1) {
				addMarkedDoor(room, ObjectTypes.PATH, paths.get(0));
			} else if (paths.size() == 2) {
				addMarkedDoor(room, ObjectTypes.LEFT_PATH, paths.get(0));
				addMarkedDoor(room, ObjectTypes.RIGHT_PATH, paths.get(1));
			} else if (paths.size() == 3) {
				addMarkedDoor(room, ObjectTypes.LEFT_PATH, paths.get(0));
				addMarkedDoor(room, ObjectTypes.MIDDLE_PATH, paths.get(1));
				addMarkedDoor(room, ObjectTypes.RIGHT_PATH, paths.get(2));
			} else if (paths.size() > 3) {
				throw new IllegalStateException("Marked more than three paths in a room. This is not currently supported.");
			}
		}
	}
	
	private static void addMarkedDoor(Room room, ObjectType type, DoorMark rightPath) {
		room.addObject(new Door(type, rightPath.destination, rightPath.property), rightPath.coord);
	}
	
	private void verifyPosition(Position pos) {
		if (!rooms.contains(pos.room()))
			throw new IllegalArgumentException("Illegal position: room is not of this location!");
	}
}