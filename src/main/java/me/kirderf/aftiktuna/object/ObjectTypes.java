package me.kirderf.aftiktuna.object;

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
	
	public static final ObjectType DOOR = new ObjectType('^', "door");
	public static final ObjectType LEFT_DOOR = new ObjectType('<', "left door");
	public static final ObjectType RIGHT_DOOR = new ObjectType('>', "right door");
	public static final ObjectType SHIP_ENTRANCE = new ObjectType('v', "ship entrance");
	public static final ObjectType SHIP_EXIT = new ObjectType('^', "ship exit");
	
	public static final Collection<CreatureType> CREATURES = List.of(EYESAUR, GOBLIN, AZURECLOPS);
	public static final Collection<ObjectType> ITEMS = List.of(FUEL_CAN, CROWBAR, BLOWTORCH, KEYCARD, KNIFE);
	public static final Collection<WeaponType> WEAPONS = List.of(CROWBAR, KNIFE);
	public static final Collection<ObjectType> DOORS = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR, SHIP_ENTRANCE, SHIP_EXIT);
}