package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.door.DoorProperty;

public class ItemType extends ObjectType {
	private final int price;
	private final String examineText;
	private final DoorProperty.Method forceMethod;
	
	public ItemType(char symbol, String name, int price, String examineText) {
		this(symbol, name, price, null, examineText);
	}
	
	public ItemType(char symbol, String name, int price, DoorProperty.Method forceMethod, String examineText) {
		super(symbol, name);
		this.price = price;
		this.forceMethod = forceMethod;
		this.examineText = examineText;
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
