package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.level.object.ObjectType;
import me.kirderf.aftiktuna.level.object.entity.Entity;

public abstract class GameObject {
	private final ObjectType type;
	private final int weight;
	
	private Position position;
	
	public GameObject(ObjectType type, int weight) {
		this.type = type;
		this.weight = weight;
	}
	
	public ObjectType getType() {
		return type;
	}
	
	public int getWeight() {
		return weight;
	}
	
	public final Room getRoom() {
		return position.room();
	}
	
	public final int getCoord() {
		return position.coord();
	}
	
	public final Position getPosition() {
		return position;
	}
	
	final void setRoom(Position pos) {
		if (this.position != null)
			throw new IllegalStateException("Position has already been set!");
		this.position = pos;
	}
	
	public final void teleport(Position pos) {
		remove();
		pos.room().addObject(this, pos);
	}
	
	public final void teleport(int coord) {
		position = position.atCoord(coord);
	}
	
	public boolean isItem() {
		return false;
	}
	
	public boolean isBlocking(Entity entity) {
		return false;
	}
	
	public final void remove() {
		getRoom().removeObject(this);
		position = null;
	}
}