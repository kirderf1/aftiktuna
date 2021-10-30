package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.object.Door;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Optional;
import java.util.function.Predicate;

public class GameObject {
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
	
	public final void moveTo(int coord) {
		moveTo(this.position.atCoord(coord));
	}
	
	public final void moveTo(Position pos) {
		if (getRoom() == pos.room())
			this.position = pos;
		else {
			remove();
			pos.room().addObject(this, pos);
		}
	}
	
	public boolean isFuelCan() {
		return false;
	}
	
	public Optional<Door> getAsDoor() {
		return Optional.empty();
	}
	
	public final Optional<GameObject> findNearest(Predicate<GameObject> predicate) {
		return findNearest(OptionalFunction.of(predicate));
	}
	
	public final <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().findNearest(mapper, getCoord());
	}
	
	public final void remove() {
		getRoom().removeObject(this);
		position = null;
	}
}