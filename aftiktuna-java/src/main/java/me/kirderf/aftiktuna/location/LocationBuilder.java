package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.*;

public final class LocationBuilder {
	private final List<Area> areas = new ArrayList<>();
	private final Map<String, DoorStatus> doorPairs = new HashMap<>();
	private Position entryPos = null;
	
	public void newArea(String label, List<String> objects, Map<Character, SymbolType> symbols) {
		Area area = new Area(label, objects.size());
		areas.add(area);
		handleSymbols(area, objects, symbols);
	}
	
	public void newTestRoom(List<String> objects, Map<Character, SymbolType> symbols) {
		newArea("Room", objects, symbols);
	}
	
	public void doorPairs(String... pairIds) {
		for(String pairId : pairIds) {
			this.doorPairs.put(pairId, new DoorStatus(DoorProperty.EMPTY));
		}
	}
	
	public void doorPair(String pairId, DoorProperty property) {
		this.doorPairs.put(pairId, new DoorStatus(property));
	}
	
	public void createDoors(DoorType type1, Position pos1, DoorType type2, Position pos2, DoorProperty property) {
		verifyPosition(pos1);
		verifyPosition(pos2);
		Location.createDoors(type1, pos1, type2, pos2, property);
	}
	
	void addToDoorPair(String pairId, Position pos, DoorType type) {
		this.doorPairs.get(pairId).addDoor(new DoorStatus.DoorInfo(pos, type));
	}
	
	public Location build() {
		verifyDoors();
		return new Location(areas, Objects.requireNonNull(entryPos, "Entry position must be set"));
	}
	
	private void verifyDoors() {
		for (Map.Entry<String, DoorStatus> entry : this.doorPairs.entrySet()) {
			if (entry.getValue().doors.size() != 2)
				throw new IllegalStateException("Not enough doors have been placed for the door pair %s".formatted(entry.getKey()));
		}
	}
	
	private void verifyPosition(Position pos) {
		if (!areas.contains(pos.area()))
			throw new IllegalArgumentException("Illegal position: area is not of this location!");
	}
	
	private void handleSymbols(Area area, List<String> objects, Map<Character, SymbolType> symbols) {
		for (int coord = 0; coord < objects.size(); coord++) {
			for (char symbol : objects.get(coord).toCharArray()) {
				if (symbols.containsKey(symbol))
					symbols.get(symbol).handle(area.getPosAt(coord), this);
				else
					handleSymbol(area, coord, symbol);
			}
		}
	}
	
	private void handleSymbol(Area area, int coord, char symbol) {
		switch (symbol) {
			case 'v' -> setEntryPos(area.getPosAt(coord));
			case 'f' -> area.addItem(ObjectTypes.FUEL_CAN, coord);
			case 'c' -> area.addItem(ObjectTypes.CROWBAR, coord);
			case 'b' -> area.addItem(ObjectTypes.BLOWTORCH, coord);
			case 'k' -> area.addItem(ObjectTypes.KEYCARD, coord);
			case 'K' -> area.addItem(ObjectTypes.KNIFE, coord);
			case 'B' -> area.addItem(ObjectTypes.BAT, coord);
			case 's' -> area.addItem(ObjectTypes.SWORD, coord);
			case 'm' -> area.addItem(ObjectTypes.METEOR_CHUNK, coord);
			case 'a' -> area.addItem(ObjectTypes.ANCIENT_COIN, coord);
			case 'G' -> area.addCreature(ObjectTypes.GOBLIN, coord);
			case 'E' -> area.addCreature(ObjectTypes.EYESAUR, coord);
			case 'Z' -> area.addCreature(ObjectTypes.AZURECLOPS, coord);
			default -> throw new RuntimeException("Unknown symbol " + symbol);
		}
	}
	
	private void setEntryPos(Position pos) {
		if (this.entryPos != null)
			throw new IllegalStateException("Entry position has already been set");
		this.entryPos = pos;
	}
	
	private class DoorStatus {
		private record DoorInfo(Position pos, DoorType type) {
		}
		
		private final DoorProperty property;
		private final List<DoorInfo> doors = new ArrayList<>(2);
		
		private DoorStatus(DoorProperty property) {
			this.property = property;
		}
		
		void addDoor(DoorInfo doorInfo) {
			if (doors.size() >= 2)
				throw new RuntimeException("Adding more than two doors of the same pair.");
			
			doors.add(doorInfo);
			if (doors.size() == 2) {
				DoorInfo doorInfo2 = doors.get(0);
				createDoors(doorInfo.type, doorInfo.pos, doorInfo2.type, doorInfo2.pos, property);
			}
		}
	}
}