package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.door.DoorProperty;

public class ItemType extends ObjectType {
	protected final String pluralName;
	private final int price;
	private final String examineText;
	private final DoorProperty.Method forceMethod;
	
	public ItemType(char symbol, String name, String pluralName, int price, String examineText) {
		this(symbol, name, pluralName, price, null, examineText);
	}
	
	public ItemType(char symbol, String name, String pluralName, int price, DoorProperty.Method forceMethod, String examineText) {
		super(symbol, name);
		this.pluralName = pluralName;
		this.price = price;
		this.forceMethod = forceMethod;
		this.examineText = examineText;
	}
	
	public String pluralName() {
		return pluralName;
	}
	
	public int getPrice() {
		return price;
	}
	
	public String getExamineText() {
		return examineText;
	}
	
	public DoorProperty.Method getForceMethod() {
		return forceMethod;
	}
}
