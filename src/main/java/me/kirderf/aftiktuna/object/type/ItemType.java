package me.kirderf.aftiktuna.object.type;

public class ItemType extends ObjectType {
	private final int price;
	private final String examineText;
	
	public ItemType(char symbol, String name, int price, String examineText) {
		super(symbol, name);
		this.price = price;
		this.examineText = examineText;
	}
	
	public int getPrice() {
		return price;
	}
	
	public String getExamineText() {
		return examineText;
	}
}
