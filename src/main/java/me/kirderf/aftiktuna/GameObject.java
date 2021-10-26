package me.kirderf.aftiktuna;

public class GameObject {
	private final char symbol;
	private final String name;
	
	public GameObject(char symbol, String name) {
		this.symbol = symbol;
		this.name = name;
	}
	
	public char getSymbol() {
		return symbol;
	}
	
	public String getName() {
		return name;
	}
}