package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Locale;
import java.util.Optional;
import java.util.function.Predicate;

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
	
	public final boolean moveTo(Position pos) {
		if (getRoom() == pos.room()) {
			if (pos.coord() != this.getCoord()) {
				Optional<GameObject> blocking = getRoom().findBlockingInRange(getPosition().getPosTowards(pos.coord()).coord(), pos.coord());
				if (blocking.isPresent()) {
					System.out.printf("The %s is blocking the way.%n", blocking.get().getType().name().toLowerCase(Locale.ROOT));
					return false;
				} else {
					this.position = pos;
					return true;
				}
			} else return true;
		} else {
			remove();
			pos.room().addObject(this, pos);
			return true;
		}
	}
	
	public boolean isItem() {
		return false;
	}
	
	public boolean isBlocking() {
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