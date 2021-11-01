package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

import java.util.Locale;

public class ObjectType {
	protected final char symbol;
	protected final String name;
	
	public ObjectType(char symbol, String name) {
		this.symbol = symbol;
		this.name = name;
	}
	
	public boolean matching(GameObject object) {
		return object.getType() == this;
	}
	
	public final char symbol() {
		return symbol;
	}
	
	public final String name() {
		return name;
	}
	
	public final String lowerCaseName() {
		return name.toLowerCase(Locale.ROOT);
	}
	
	@Override
	public String toString() {
		return "ObjectType[" +
				"symbol=" + symbol + ", " +
				"name=" + name + ']';
	}
	
}
