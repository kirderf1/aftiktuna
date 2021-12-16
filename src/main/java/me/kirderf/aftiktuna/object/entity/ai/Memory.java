package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorProperty;

import java.util.HashMap;
import java.util.Map;

public final class Memory {
	private final Map<Identifier, AreaMemory> areaMap = new HashMap<>();
	
	private AreaMemory getOrCreateMemory(Area area) {
		return areaMap.computeIfAbsent(area.getId(), id -> new AreaMemory());
	}
	
	public void observeDoorProperty(Door door, DoorProperty property) {
		getOrCreateMemory(door.getArea()).observedDoorProperties.put(door.getId(), property);
	}
	
	public DoorProperty getObservedProperty(Door door) {
		return getOrCreateMemory(door.getArea()).observedDoorProperties.get(door.getId());
	}
	
	private static final class AreaMemory {
		private final Map<Identifier, DoorProperty> observedDoorProperties = new HashMap<>();
	}
}
