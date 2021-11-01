package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

import java.util.Collection;
import java.util.List;
import java.util.Locale;

public record ObjectType(char symbol, String name) {
	public static final ObjectType AFTIK = new ObjectType('A', "Aftik");
	public static final ObjectType CREATURE = new ObjectType('C', "Creature");
	
	public static final ObjectType FUEL_CAN = new ObjectType('f', "Fuel can");
	public static final ObjectType CROWBAR = new ObjectType('c', "Crowbar");
	public static final ObjectType BLOWTORCH = new ObjectType('b', "Blowtorch");
	public static final ObjectType KEYCARD = new ObjectType('k', "Keycard");
	public static final ObjectType KNIFE = new ObjectType('K', "Knife");
	
	public static final ObjectType DOOR = new ObjectType('^', "Door");
	public static final ObjectType LEFT_DOOR = new ObjectType('<', "Left door");
	public static final ObjectType RIGHT_DOOR = new ObjectType('>', "Right door");
	
	public static final Collection<ObjectType> CREATURES = List.of(CREATURE);
	public static final Collection<ObjectType> ITEMS = List.of(FUEL_CAN, CROWBAR, BLOWTORCH, KEYCARD, KNIFE);
	public static final Collection<ObjectType> WEAPONS = List.of(CROWBAR, KNIFE);
	public static final Collection<ObjectType> DOORS = List.of(DOOR, LEFT_DOOR, RIGHT_DOOR);
	
	public boolean matching(GameObject object) {
		return object.getType() == this;
	}
	
	public String lowerCaseName() {
		return name.toLowerCase(Locale.ROOT);
	}
}
