package me.kirderf.aftiktuna.object;

public class ItemType extends ObjectType {
	private final int price;
	
	public ItemType(char symbol, String name, int price) {
		super(symbol, name);
		this.price = price;
	}
	
	public int getPrice() {
		return price;
	}
}
