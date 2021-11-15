package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.door.DoorType;

import java.util.*;
import java.util.concurrent.atomic.AtomicReference;

public final class LocationBuilder {
	private final List<Area> areas = new ArrayList<>();
	private final Map<Area, List<DoorMark>> doorMap = new HashMap<>();
	private final Map<Area, List<DoorMark>> pathMap = new HashMap<>();
	
	public Area newArea(String label, int size) {
		Area area = new Area(label, size);
		areas.add(area);
		doorMap.put(area, new ArrayList<>());
		pathMap.put(area, new ArrayList<>());
		return area;
	}
	
	public Area newTestRoom(int size) {
		return newArea("Room", size);
	}
	
	public void markDoors(Position pos1, Position pos2) {
		markDoors(pos1, pos2, DoorProperty.EMPTY);
	}
	
	public void markDoors(Position pos1, Position pos2, DoorProperty property) {
		AtomicReference<DoorProperty> reference = new AtomicReference<>(property);
		doorMap.get(pos1.area()).add(new DoorMark(pos1.coord(), pos2, reference));
		doorMap.get(pos2.area()).add(new DoorMark(pos2.coord(), pos1, reference));
	}
	
	public void markPath(Position pos1, Position pos2) {
		AtomicReference<DoorProperty> reference = new AtomicReference<>(DoorProperty.EMPTY);
		pathMap.get(pos1.area()).add(new DoorMark(pos1.coord(), pos2, reference));
		pathMap.get(pos2.area()).add(new DoorMark(pos2.coord(), pos1, reference));
	}
	
	private static record DoorMark(int coord, Position destination, AtomicReference<DoorProperty> property) {}
	
	public void createDoors(DoorType type1, Position pos1, DoorType type2, Position pos2) {
		createDoors(type1, pos1, type2, pos2, DoorProperty.EMPTY);
	}
	
	public void createDoors(DoorType type1, Position pos1, DoorType type2, Position pos2, DoorProperty property) {
		verifyPosition(pos1);
		verifyPosition(pos2);
		Location.createDoors(type1, pos1, type2, pos2, property);
	}
	
	public Location build(Position entryPos) {
		verifyPosition(entryPos);
		buildDoors();
		return new Location(areas, entryPos);
	}
	
	private void buildDoors() {
		for (Area area : areas) {
			List<DoorMark> marks = doorMap.get(area);
			marks.sort(Comparator.comparingInt(DoorMark::coord));
			if (marks.size() == 1) {
				addMarkedDoor(area, ObjectTypes.DOOR, marks.get(0));
			} else if (marks.size() == 2) {
				addMarkedDoor(area, ObjectTypes.LEFT_DOOR, marks.get(0));
				addMarkedDoor(area, ObjectTypes.RIGHT_DOOR, marks.get(1));
			} else if (marks.size() == 3) {
				addMarkedDoor(area, ObjectTypes.LEFT_DOOR, marks.get(0));
				addMarkedDoor(area, ObjectTypes.MIDDLE_DOOR, marks.get(1));
				addMarkedDoor(area, ObjectTypes.RIGHT_DOOR, marks.get(2));
			} else if (marks.size() > 3) {
				throw new IllegalStateException("Marked more than three doors in an area. This is not currently supported.");
			}
			List<DoorMark> paths = pathMap.get(area);
			paths.sort(Comparator.comparingInt(DoorMark::coord));
			if (paths.size() == 1) {
				addMarkedDoor(area, ObjectTypes.PATH, paths.get(0));
			} else if (paths.size() == 2) {
				addMarkedDoor(area, ObjectTypes.LEFT_PATH, paths.get(0));
				addMarkedDoor(area, ObjectTypes.RIGHT_PATH, paths.get(1));
			} else if (paths.size() == 3) {
				addMarkedDoor(area, ObjectTypes.LEFT_PATH, paths.get(0));
				addMarkedDoor(area, ObjectTypes.MIDDLE_PATH, paths.get(1));
				addMarkedDoor(area, ObjectTypes.RIGHT_PATH, paths.get(2));
			} else if (paths.size() > 3) {
				throw new IllegalStateException("Marked more than three paths in an area. This is not currently supported.");
			}
		}
	}
	
	private static void addMarkedDoor(Area area, DoorType type, DoorMark rightPath) {
		area.addObject(new Door(type, rightPath.destination, rightPath.property), rightPath.coord);
	}
	
	private void verifyPosition(Position pos) {
		if (!areas.contains(pos.area()))
			throw new IllegalArgumentException("Illegal position: area is not of this location!");
	}
}