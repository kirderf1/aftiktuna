package me.kirderf.aftiktuna.object.door;

import me.kirderf.aftiktuna.object.ObjectType;

public class DoorType extends ObjectType {
	private final String categoryName;
	
	public DoorType(char symbol, String name) {
		super(symbol, name);
		categoryName = name;
	}
	
	public DoorType(char symbol, String name, DoorType parent) {
		super(symbol, name);
		categoryName = parent.name();
	}
	
	public String getCategoryName() {
		return categoryName;
	}
}