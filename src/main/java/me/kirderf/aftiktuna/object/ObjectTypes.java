package me.kirderf.aftiktuna.object;

import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.Stats;

import java.util.Collection;
import java.util.List;

public final class ObjectTypes {
	public static final ObjectType AFTIK = new ObjectType('A', "aftik");
	
	public static final CreatureType EYESAUR = new CreatureType('E', "Eyesaur", new Stats(7, 7, 4));
	public static final CreatureType GOBLIN = new CreatureType('G', "Goblin", new Stats(2, 4, 10));
	public static final CreatureType AZURECLOPS = new CreatureType('Z', "Azureclops", new Stats(15, 10, 4));
	
	public static final ObjectType FUEL_CAN = new ObjectType('f', "fuel can");
	public static final WeaponType CROWBAR = new WeaponType('c', "crowbar", 3);
	public static final ObjectType BLOWTORCH = new ObjectType('b', "blowtorch");
	public static final ObjectType KEYCARD = new ObjectType('k', "keycard");
	public static final WeaponType KNIFE = new WeaponType('K', "knife", 4);
	public static final WeaponType SWORD = new WeaponType('s', "sword", 5);
	
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
	public static final Collection<ObjectType> ITEMS = List.of(FUEL_CAN, CROWBAR, BLOWTORCH, KEYCARD, KNIFE, SWORD);
	public static final Collection<WeaponType> WEAPONS = List.of(CROWBAR, KNIFE, SWORD);
	public static final Collection<DoorType> DOORS = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR, MIDDLE_DOOR,
			SHIP_ENTRANCE, SHIP_EXIT, PATH, LEFT_PATH, RIGHT_PATH, MIDDLE_PATH);
	public static final Collection<DoorType> FORCEABLE = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR, MIDDLE_DOOR,
			SHIP_ENTRANCE, SHIP_EXIT);
}