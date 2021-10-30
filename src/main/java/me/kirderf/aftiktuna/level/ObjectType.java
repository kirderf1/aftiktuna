package me.kirderf.aftiktuna.level;

public record ObjectType(char symbol, String name) {
	public static final ObjectType AFTIK = new ObjectType('A', "Aftik");
	public static final ObjectType FUEL_CAN = new ObjectType('f', "Fuel can");
	public static final ObjectType DOOR = new ObjectType('^', "Door");
}
