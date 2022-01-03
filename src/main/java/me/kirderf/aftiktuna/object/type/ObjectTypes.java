package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.door.DoorProperty;
import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.Stats;

import java.util.Collection;
import java.util.List;

public final class ObjectTypes {
	public static final ObjectType AFTIK = new ObjectType('A', "aftik");
	public static final ObjectType SHOPKEEPER = new ObjectType('S', "shopkeeper");
	
	public static final CreatureType EYESAUR = new CreatureType('E', "Eyesaur", new Stats(7, 7, 4), "A quadruped that has their one eye on the side of their tail. Dangerous.");
	public static final CreatureType GOBLIN = new CreatureType('G', "Goblin", new Stats(2, 4, 10), "A bat-like creature that uses their arms to move. Dangerous in packs.");
	public static final CreatureType AZURECLOPS = new CreatureType('Z', "Azureclops", new Stats(15, 10, 4), "A large hulking blue cyclops. Very dangerous.");
	
	public static final ItemType FUEL_CAN = new ItemType('f', "fuel can", "fuel cans", 3500, "Needed in order to travel to your next location.");
	public static final WeaponType CROWBAR = new WeaponType('c', "crowbar", "crowbars", 3, -1, DoorProperty.Method.FORCE, "Can be used to force open certain doors. Can also be used as an improvised weapon.");
	public static final ItemType BLOWTORCH = new ItemType('b', "blowtorch", "blowtorches", -1, DoorProperty.Method.CUT, "Can be used to cut open doors.");
	public static final ItemType KEYCARD = new ItemType('k', "keycard", "keycards", -1, "Can be used to enter certain doors.");
	public static final WeaponType KNIFE = new WeaponType('K', "knife", "knives", 3, 300, "Can be used as an improvised weapon.");
	public static final WeaponType BAT = new WeaponType('B', "bat", "bats", 4, 1000, "Can be used as a weapon.");
	public static final WeaponType SWORD = new WeaponType('s', "sword", "swords", 5, 3000, "Can be used as a decent weapon.");
	public static final ItemType METEOR_CHUNK = new ItemType('m', "meteor chunk", "meteor chunks", 2500, "Not useful beyond being sold for points.");
	public static final ItemType ANCIENT_COIN = new ItemType('a', "ancient coin", "ancient coins", 500, "Not useful beyond being sold for points.");
	public static final ItemType MEDKIT = new ItemType('+', "medkit", "medkits", -1, "Can be used to recover some health.");
	
	public static final DoorType DOOR = new DoorType('^', "door");
	public static final DoorType LEFT_DOOR = new DoorType('<', "left door", DOOR);
	public static final DoorType RIGHT_DOOR = new DoorType('>', "right door", DOOR);
	public static final DoorType MIDDLE_DOOR = new DoorType('^', "Middle door", DOOR);
	public static final DoorType SHIP_ENTRANCE = new DoorType('v', "ship entrance", DOOR);
	public static final DoorType SHIP_EXIT = new DoorType('^', "ship exit", DOOR);
	public static final DoorType PATH = new DoorType('^', "path");
	public static final DoorType LEFT_PATH = new DoorType('<', "left path", PATH);
	public static final DoorType RIGHT_PATH = new DoorType('>', "right path", PATH);
	public static final DoorType MIDDLE_PATH = new DoorType('^', "Middle path", PATH);
	
	public static final Collection<CreatureType> CREATURES = List.of(EYESAUR, GOBLIN, AZURECLOPS);
	public static final Collection<ItemType> ITEMS = List.of(FUEL_CAN, CROWBAR, BLOWTORCH, KEYCARD, KNIFE, BAT, SWORD, METEOR_CHUNK, ANCIENT_COIN, MEDKIT);
	public static final Collection<WeaponType> WEAPONS = List.of(CROWBAR, KNIFE, BAT, SWORD);
	public static final Collection<DoorType> DOORS = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR, MIDDLE_DOOR,
			SHIP_ENTRANCE, SHIP_EXIT, PATH, LEFT_PATH, RIGHT_PATH, MIDDLE_PATH);
	public static final Collection<DoorType> FORCEABLE = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR, MIDDLE_DOOR,
			SHIP_ENTRANCE, SHIP_EXIT);
}