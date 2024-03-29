package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorPairInfo;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.door.DoorType;

import java.util.List;

public final class Location {
	private final List<Area> areas;
	private final Position entryPos;
	
	Location(List<Area> areas, Position entryPos) {
		this.areas = List.copyOf(areas);
		this.entryPos = entryPos;
	}
	
	public List<Area> getAreas() {
		return areas;
	}
	
	public Position getEntryPos() {
		return entryPos;
	}
	
	public void addAtEntry(GameObject object) {
		entryPos.area().addObject(object, entryPos);
	}
	
	static void createDoors(DoorType type1, Position pos1, DoorType type2, Position pos2, DoorProperty property) {
		DoorPairInfo pairInfo = new DoorPairInfo(property);
		pos1.area().addObject(new Door(type1, pos2, pairInfo), pos1.coord());
		pos2.area().addObject(new Door(type2, pos1, pairInfo), pos2.coord());
	}
}