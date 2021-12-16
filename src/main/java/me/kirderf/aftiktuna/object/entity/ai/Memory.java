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
	
	public void observeDoorEntryFailure(Door door, DoorProperty.FailureType failure) {
		getOrCreateMemory(door.getArea()).observedDoorFailures.put(door.getId(), failure);
	}
	
	public void observeDoorForceSuccess(Door door) {
		getOrCreateMemory(door.getArea()).observedDoorFailures.remove(door.getId());
	}
	
	public DoorProperty.FailureType getObservedFailureType(Door door) {
		return getOrCreateMemory(door.getArea()).observedDoorFailures.get(door.getId());
	}
	
	private static final class AreaMemory {
		private final Map<Identifier, DoorProperty.FailureType> observedDoorFailures = new HashMap<>();
	}
}
