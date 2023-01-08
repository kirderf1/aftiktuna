package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.SymbolType;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;
import java.util.Map;

@SuppressWarnings("unused")
public final class EarlyTestingLocations {
	
	public static Location createLocation1() {
		LocationBuilder builder = new LocationBuilder();
		builder.newTestRoom(List.of("", "v", "", "", "f"), Map.of());
		return builder.build();
	}
	
	public static Location createLocation2() {
		LocationBuilder builder = new LocationBuilder();
		builder.newTestRoom(List.of("f", "v", "", "f"), Map.of());
		return builder.build();
	}
	
	public static Location createLocation3() {
		LocationBuilder builder = new LocationBuilder();
		builder.newTestRoom(List.of("v", "", "ff"), Map.of());
		return builder.build();
	}
	
	public static Location createDoorLocation1() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("door");
		builder.newTestRoom(List.of("v", "", "^"),
				Map.of('^', new SymbolType.Door("door", ObjectTypes.DOOR)));
		builder.newTestRoom(List.of("^", "", "f"),
				Map.of('^', new SymbolType.Door("door", ObjectTypes.DOOR)));
		return builder.build();
	}
	
	public static Location createDoorLocation2() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("left", "right", "side");
		builder.newTestRoom(List.of("v", "<", ">"),
				Map.of('<', new SymbolType.Door("left", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("right", ObjectTypes.RIGHT_DOOR)));
		builder.newTestRoom(List.of("<", "", ">"),
				Map.of('<', new SymbolType.Door("left", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("side", ObjectTypes.RIGHT_DOOR)));
		builder.newTestRoom(List.of("<", ">", "ff"),
				Map.of('<', new SymbolType.Door("side", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("right", ObjectTypes.RIGHT_DOOR)));
		return builder.build();
	}
	
	public static Location createToolsLocation() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPair("stuck", DoorProperty.STUCK);
		builder.doorPair("sealed", DoorProperty.SEALED);
		builder.doorPair("locked", DoorProperty.LOCKED);
		builder.newTestRoom(List.of("v", "<", "ck", ">"),
				Map.of('<', new SymbolType.Door("stuck", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("sealed", ObjectTypes.RIGHT_DOOR)));
		builder.newTestRoom(List.of("<b", ">"),
				Map.of('<', new SymbolType.Door("locked", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("stuck", ObjectTypes.RIGHT_DOOR)));
		builder.newTestRoom(List.of("^", "", "f"),
				Map.of('^', new SymbolType.Door("locked", ObjectTypes.DOOR)));
		builder.newTestRoom(List.of("^", "", "f"),
				Map.of('^', new SymbolType.Door("sealed", ObjectTypes.DOOR)));
		return builder.build();
	}
	
	public static Location createBlockingLocation() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("left", "right");
		builder.newTestRoom(List.of("vc", "<", "Ef", ">", "", "fk"),
				Map.of('<', new SymbolType.Door("left", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("right", ObjectTypes.RIGHT_DOOR),
						'E', new SymbolType.ImmovableCreature(ObjectTypes.EYESAUR)));
		builder.newTestRoom(List.of("<", "K", ">"),
				Map.of('<', new SymbolType.Door("left", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("right", ObjectTypes.RIGHT_DOOR)));
		return builder.build();
	}
	
	public static Location createDeathLocation() {
		LocationBuilder builder = new LocationBuilder();
		builder.newTestRoom(List.of("v", "", "ZZ", "Z", "f"), Map.of());
		return builder.build();
	}
}