package me.kirderf.aftiktuna.location.levels;

import me.kirderf.aftiktuna.location.Location;
import me.kirderf.aftiktuna.location.LocationBuilder;
import me.kirderf.aftiktuna.location.SymbolType;
import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.entity.Stats;
import me.kirderf.aftiktuna.object.type.ObjectTypes;

import java.util.List;
import java.util.Map;

public final class Locations {
	static final List<LocationCategory> categories = List.of(
			new LocationCategory("abandoned facility", List.of(Locations::abandonedFacility, Locations::abandonedFacility2)),
			new LocationCategory("forest", List.of(Locations::goblinForest, Locations::eyesaurForest)),
			new LocationCategory("village", List.of(Locations::village)));
	
	public static void checkLocations() {
		categories.forEach(LocationCategory::checkLocations);
	}
	
	private static Location abandonedFacility() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("path", "corridor1", "corridor2", "room1", "room3");
		builder.doorPair("entrance", DoorProperty.LOCKED);
		builder.doorPair("sealed", DoorProperty.SEALED);
		builder.doorPair("room2", DoorProperty.STUCK);
		builder.newArea("Field in front of a building", List.of("v", "", "^", "", "", ">"),
				Map.of('^', new SymbolType.Door("entrance", ObjectTypes.DOOR),
						'>', new SymbolType.Door("path", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Field", List.of("^", "", "", "k", ""),
				Map.of('^', new SymbolType.Door("path", ObjectTypes.PATH)));
		builder.newArea("Entrance hall", List.of("", "<", "", "^", "", ">", ""),
				Map.of('<', new SymbolType.Door("sealed", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("corridor1", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("entrance", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Corridor", List.of("<", "", "^", "E", ">"),
				Map.of('<', new SymbolType.Door("corridor1", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("room1", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("corridor2", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Corridor", List.of("<", "", "^", "", ">"),
				Map.of('<', new SymbolType.Door("corridor2", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("room2", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("room3", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Room", List.of("", "c", "", "^"),
				Map.of('^', new SymbolType.Door("room1", ObjectTypes.DOOR)));
		builder.newArea("Room", List.of("b", "", "", "^"),
				Map.of('^', new SymbolType.Door("room2", ObjectTypes.DOOR)));
		builder.newArea("Room", List.of("^", "E", "", "f"),
				Map.of('^', new SymbolType.Door("room3", ObjectTypes.DOOR)));
		builder.newArea("Room", List.of("ff", "Z", "^", "s"),
				Map.of('^', new SymbolType.Door("sealed", ObjectTypes.DOOR)));
		
		return builder.build();
	}
	
	private static Location abandonedFacility2() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("path", "entrance", "corridor1", "corridor2", "room1", "room2", "room3");
		builder.doorPair("side", DoorProperty.STUCK);
		builder.doorPair("storage", DoorProperty.SEALED);
		builder.newArea("Field in front of a building", List.of("<", "", "", "^", "", "v"),
				Map.of('<', new SymbolType.Door("path", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("entrance", ObjectTypes.DOOR)));
		builder.newArea("Field next to a building", List.of("^", "", "", "", ">"),
				Map.of('^', new SymbolType.Door("side", ObjectTypes.DOOR),
						'>', new SymbolType.Door("path", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Entrance hall", List.of("<", "E", "", "^", "a", ">"),
				Map.of('<', new SymbolType.Door("corridor1", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("entrance", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("room1", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Corridor", List.of("<", "E", "K", "^", "", ">"),
				Map.of('<', new SymbolType.Door("corridor1", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("corridor2", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("room2", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Corridor", List.of("<", "", "^", "", ">"),
				Map.of('<', new SymbolType.Door("side", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("corridor2", ObjectTypes.MIDDLE_DOOR),
						'>', new SymbolType.Door("room3", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Room", List.of("^", "", "Z", "bs"),
				Map.of('^', new SymbolType.Door("room1", ObjectTypes.DOOR)));
		builder.newArea("Room", List.of("^", "", "m", "c"),
				Map.of('^', new SymbolType.Door("room2", ObjectTypes.DOOR)));
		builder.newArea("Room", List.of("<", "G", "f", ">"),
				Map.of('<', new SymbolType.Door("room3", ObjectTypes.LEFT_DOOR),
						'>', new SymbolType.Door("storage", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Storage Room", List.of("^", "aa", "ffa"),
				Map.of('^', new SymbolType.Door("storage", ObjectTypes.DOOR)));
		
		return builder.build();
	}
	
	private static Location goblinForest() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("entrance", "path1", "path2", "path3", "path4", "path5");
		builder.doorPair("shack", DoorProperty.STUCK);
		builder.newArea("Field in front of a forest", List.of("", "v", "", "", "^", "", ""),
				Map.of('^', new SymbolType.Door("entrance", ObjectTypes.PATH)));
		builder.newArea("Forest entrance", List.of("<", "", "^", "", ">"),
				Map.of('<', new SymbolType.Door("path1", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("entrance", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("path5", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path leading to a shack", List.of("^", "", "<", "", ">", ""),
				Map.of('^', new SymbolType.Door("shack", ObjectTypes.DOOR),
						'<', new SymbolType.Door("path2", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("path1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "", "", "G", ">"),
				Map.of('<', new SymbolType.Door("path2", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("path3", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "", ">", "", "G", "c"),
				Map.of('<', new SymbolType.Door("path4", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("path5", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "B", "G", "G", "", ">"),
				Map.of('<', new SymbolType.Door("path3", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("path4", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Shack", List.of("af", "fE", "", "^"),
				Map.of('^', new SymbolType.Door("shack", ObjectTypes.DOOR)));
		
		return builder.build();
	}
	
	private static Location eyesaurForest() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("left_entrance", "right_entrance", "left1", "left2", "mid_left1", "mid_left2",
				"right1", "right2", "mid_right1", "mid_right2", "mid");
		builder.newArea("Field in front of a forest", List.of("", "<", "", "v", "", "", ">", ""),
				Map.of('<', new SymbolType.Door("left_entrance", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("right_entrance", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest entrance", List.of("<", "E", "", "^", "", ">"),
				Map.of('<', new SymbolType.Door("left1", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("left_entrance", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("mid_left1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "", ">", "", "f", ""),
				Map.of('<', new SymbolType.Door("left2", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("left1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "E", "a", ">"),
				Map.of('<', new SymbolType.Door("left2", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("mid_left2", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest entrance", List.of("<", "", "^", "K", ">", ""),
				Map.of('<', new SymbolType.Door("mid_right1", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("right_entrance", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("right1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "fE", "^", "", ">"),
				Map.of('<', new SymbolType.Door("mid_right2", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("right2", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("right1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("^", "m", "E", "E", "bf"),
				Map.of('^', new SymbolType.Door("right2", ObjectTypes.PATH)));
		builder.newArea("Forest path", List.of("<", "", "^", "E", "", ">"),
				Map.of('<', new SymbolType.Door("mid_left1", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("mid", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("mid_right1", ObjectTypes.RIGHT_PATH)));
		builder.newArea("Forest path", List.of("<", "", "B", "^", "", ">", ""),
				Map.of('<', new SymbolType.Door("mid_left2", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("mid", ObjectTypes.MIDDLE_PATH),
						'>', new SymbolType.Door("mid_right2", ObjectTypes.RIGHT_PATH)));
		
		return builder.build();
	}
	
	private static Location village() {
		LocationBuilder builder = new LocationBuilder();
		builder.doorPairs("road", "side", "aftik_house", "store", "side_house");
		builder.doorPair("stuck_house", DoorProperty.STUCK);
		builder.newArea("Village road", List.of("", "<", "", "v", "", "^", "", "", ">", ""),
				Map.of('<', new SymbolType.Door("aftik_house", ObjectTypes.LEFT_DOOR),
						'^', new SymbolType.Door("road", ObjectTypes.PATH),
						'>', new SymbolType.Door("store", ObjectTypes.RIGHT_DOOR)));
		builder.newArea("Village road", List.of("", "<", "", "", "", ">", "", "", "^", ""),
				Map.of('<', new SymbolType.Door("side", ObjectTypes.LEFT_PATH),
						'>', new SymbolType.Door("road", ObjectTypes.RIGHT_PATH),
						'^', new SymbolType.Door("stuck_house", ObjectTypes.DOOR)));
		builder.newArea("Side path", List.of("a", "<", "", "G", "G", "", "G", "^", ""),
				Map.of('<', new SymbolType.Door("side", ObjectTypes.LEFT_PATH),
						'^', new SymbolType.Door("side_house", ObjectTypes.DOOR)));
		builder.newArea("House", List.of("A", "", "^", ""),
				Map.of('^', new SymbolType.Door("aftik_house", ObjectTypes.DOOR),
						'A', new SymbolType.Recruitable("Plum", new Stats(10, 2, 9))));
		builder.newArea("Store", List.of("^", "", "S", ""),
				Map.of('^', new SymbolType.Door("store", ObjectTypes.DOOR),
						'S', new SymbolType.Shop(ObjectTypes.FUEL_CAN, ObjectTypes.BAT, ObjectTypes.SWORD)));
		builder.newArea("House", List.of("^", "", "m", "a"),
				Map.of('^', new SymbolType.Door("stuck_house", ObjectTypes.DOOR)));
		builder.newArea("House", List.of("^", "", "m", "B"),
				Map.of('^', new SymbolType.Door("side_house", ObjectTypes.DOOR)));
		
		return builder.build();
	}
}
