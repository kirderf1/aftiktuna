package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.entity.Entity;

public abstract class GameObject {
	private final ObjectType type;
	private final int weight;
	
	private Position position;
	
	public GameObject(ObjectType type, int weight) {
		this.type = type;
		this.weight = weight;
	}
	
	public final ObjectType getType() {
		return type;
	}
	
	public boolean hasCustomName() {
		return false;
	}
	
	public char getDisplaySymbol() {
		return type.symbol();
	}
	
	public String getDisplayName(boolean definite, boolean capitalized) {
		if (definite)
			return (capitalized ? "The " : "the ") + type.name();
		else
			return capitalized ? type.capitalizedName() : type.name();
	}
	
	public final int getWeight() {
		return weight;
	}
	
	public final Area getArea() {
		return position.area();
	}
	
	public final int getCoord() {
		return position.coord();
	}
	
	public final Position getPosition() {
		return position;
	}
	
	final void setArea(Position pos) {
		if (this.position != null)
			throw new IllegalStateException("Position has already been set!");
		this.position = pos;
	}
	
	public final void teleport(Position pos) {
		remove();
		pos.area().addObject(this, pos);
	}
	
	public final void teleport(int coord) {
		position = position.atCoord(coord);
	}
	
	public boolean isBlocking(Entity entity) {
		return false;
	}
	
	public final void remove() {
		getArea().removeObject(this);
		position = null;
	}
}