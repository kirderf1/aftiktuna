package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.SymbolType;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Stats;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;
import java.util.Map;

@SuppressWarnings("unused")
public final class CrewTestingLocations {
	public static Location separationTest() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPair("door", DoorProperty.LOCKED);
		builder.newTestRoom(List.of("^kb", "v", "", ""),
				Map.of('^', new SymbolType.Door("door", ObjectTypes.DOOR)));
		builder.newTestRoom(List.of("", "^", "", ""),
				Map.of('^', new SymbolType.Door("door", ObjectTypes.DOOR)));
		
		return builder.build();
	}
	
	public static Location recruitmentAndStore() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("path");
		builder.newTestRoom(List.of("v", "^", "amkS", "A"),
				Map.of('^', new SymbolType.Door("path", ObjectTypes.PATH),
						'A', new SymbolType.Recruitable("Plum", new Stats(10, 2, 9)),
						'S', new SymbolType.Shop(ObjectTypes.FUEL_CAN, ObjectTypes.SWORD)));
		builder.newTestRoom(List.of("", "^", "", "Z"),
				Map.of('^', new SymbolType.Door("path", ObjectTypes.PATH)));
		
		return builder.build();
	}
}
