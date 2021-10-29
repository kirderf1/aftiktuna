package me.kirderf.aftiktuna;

public class GameObject {
	private final char symbol;
	private final String name;
	private final int weight;
	
	public GameObject(char symbol, String name, int weight) {
		this.symbol = symbol;
		this.name = name;
		this.weight = weight;
	}
	
	public char getSymbol() {
		return symbol;
	}
	
	public String getName() {
		return name;
	}
	
	public int getWeight() {
		return weight;
	}
	
	public boolean isFuelCan() {
		return false;
	}
}