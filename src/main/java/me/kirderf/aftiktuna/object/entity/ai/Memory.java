package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.Reference;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.door.DoorPairInfo;
import me.kirderf.aftiktuna.object.door.DoorProperty;

import java.util.HashMap;
import java.util.Map;
import java.util.Optional;

public final class Memory {
	private final Map<Identifier<Area>, AreaMemory> areaMap = new HashMap<>();
	private final Map<Identifier<DoorPairInfo>, DoorProperty> observedDoorProperties = new HashMap<>();
	
	private AreaMemory getOrCreateMemory(Area area) {
		return areaMap.computeIfAbsent(area.getId(), AreaMemory::new);
	}
	
	public void observeDoorProperty(Door door, DoorProperty property) {
		observedDoorProperties.put(door.getPairId(), property);
	}
	
	public void observeNewConnection(Area area1, Area area2, Identifier<DoorPairInfo> doorPairId) {
		registerPath(area1, area2, doorPairId);
		registerPath(area2, area1, doorPairId);
	}
	
	public DoorProperty getObservedProperty(Door door) {
		return observedDoorProperties.getOrDefault(door.getPairId(), DoorProperty.EMPTY);
	}
	
	public boolean hasObservedProperty(Door door) {
		return observedDoorProperties.containsKey(door.getPairId());
	}
	
	public Optional<Door> findDoorTowards(Area fromArea, Identifier<Area> toArea) {
		return getOrCreateMemory(fromArea).getDirectionTo(toArea)
				.map(AreaDirection::doorRef).flatMap(ref -> ref.find(fromArea));
	}
	
	public void registerPath(Area areaFrom, Area areaTo, Identifier<DoorPairInfo> doorPairId) {
		AreaMemory fromMemory = getOrCreateMemory(areaFrom);
		AreaMemory destMemory = getOrCreateMemory(areaTo);
		Door door = areaFrom.objectStream().flatMap(Door.CAST.toStream()).filter(door_ -> door_.getPairId().equals(doorPairId)).findAny().orElseThrow();
		getOrCreateMemory(areaFrom).update(areaTo.getId(), new AreaDirection(new Reference<>(door, Door.class), 1));
		
		for (AreaMemory areaMemory : areaMap.values()) {
			areaMemory.replicatePath(areaTo.getId(), areaFrom.getId(), 1);
		}
		for (Map.Entry<Identifier<Area>, AreaDirection> entry : destMemory.directionMap.entrySet()) {
			fromMemory.replicatePath(entry.getKey(), areaTo.getId(), entry.getValue().distance());
			for (AreaMemory areaMemory : areaMap.values()) {
				areaMemory.replicatePath(entry.getKey(), areaFrom.getId(), entry.getValue().distance() + 1);
			}
		}
	}
	
	private static final class AreaMemory {
		private final Identifier<Area> areaId;
		private final Map<Identifier<Area>, AreaDirection> directionMap = new HashMap<>();
		
		private AreaMemory(Identifier<Area> areaId) {
			this.areaId = areaId;
		}
		
		private void replicatePath(Identifier<Area> newArea, Identifier<Area> replicatedArea, int extraDistance) {
			if (directionMap.containsKey(replicatedArea)) {
				AreaDirection direction = directionMap.get(replicatedArea);
				update(newArea, new AreaDirection(direction.doorRef, direction.distance + extraDistance));
			}
		}
		
		private void update(Identifier<Area> area, AreaDirection direction) {
			if (area.equals(this.areaId))
				return;
			
			AreaDirection old = directionMap.get(area);
			if (old == null || direction.distance < old.distance)
				directionMap.put(area, direction);
		}
		
		private Optional<AreaDirection> getDirectionTo(Identifier<Area> targetArea) {
			return Optional.ofNullable(directionMap.get(targetArea));
		}
		
		@Override
		public String toString() {
			return "AreaMemory{" + areaId + '}';
		}
	}
	
	private record AreaDirection(Reference<Door> doorRef, int distance) {}
}
