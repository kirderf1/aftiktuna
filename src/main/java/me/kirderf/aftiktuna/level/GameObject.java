package me.kirderf.aftiktuna.level;

import java.util.Optional;
import java.util.function.Predicate;

public class GameObject {
	private final char symbol;
	private final String name;
	private final int weight;
	
	private Room room;
	private int position;
	
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
	
	public Room getRoom() {
		return room;
	}
	
	public int getPosition() {
		return position;
	}
	
	void setRoom(Room room, int pos) {
		if (this.room != null)
			this.room.removeObject(this);
		this.room = room;
		this.position = pos;
	}
	
	public void move(int pos) {
		room.verifyValidPosition(pos);
		this.position = pos;
	}
	
	public boolean isFuelCan() {
		return false;
	}
	
	public final Optional<GameObject> findNearest(Predicate<GameObject> condition) {
		return getRoom().findNearest(condition, getPosition());
	}
	
	public final void remove() {
		getRoom().removeObject(this);
	}
}