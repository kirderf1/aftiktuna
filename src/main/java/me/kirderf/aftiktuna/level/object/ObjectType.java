package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

public class ObjectType {
	protected final char symbol;
	protected final String name, capitalizedName;
	
	public ObjectType(char symbol, String name) {
		this.symbol = symbol;
		this.name = name;
		this.capitalizedName = Character.toUpperCase(name.charAt(0)) + name.substring(1);
	}
	
	public boolean matching(GameObject object) {
		return object.getType() == this;
	}
	
	public final char symbol() {
		return symbol;
	}
	
	public final String capitalizedName() {
		return capitalizedName;
	}
	
	public final String name() {
		return name;
	}
	
	@Override
	public String toString() {
		return "ObjectType[" +
				"symbol=" + symbol + ", " +
				"name=" + name + ']';
	}
	
}
