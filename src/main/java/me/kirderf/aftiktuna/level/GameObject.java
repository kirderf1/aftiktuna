package me.kirderf.aftiktuna.level;

import me.kirderf.aftiktuna.OptionalFunction;
import me.kirderf.aftiktuna.level.object.door.Door;
import me.kirderf.aftiktuna.level.object.ObjectType;

import java.util.Comparator;
import java.util.Locale;
import java.util.Optional;

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
	
	public final boolean tryMoveTo(int coord) {
		if(coord != this.getCoord()) {
			Optional<GameObject> blocking = findBlockingTo(coord);
			if(blocking.isPresent()) {
				System.out.printf("The %s is blocking the way.%n", blocking.get().getType().name().toLowerCase(Locale.ROOT));
				return false;
			} else {
				this.position = getPosition().atCoord(coord);
				return true;
			}
		} else return true;
	}
	
	public final boolean isAccessBlocked(int coord) {
		return findBlockingTo(coord).isPresent();
	}
	
	private Optional<GameObject> findBlockingTo(int coord) {
		return getRoom().findBlockingInRange(getPosition().getPosTowards(coord).coord(), coord);
	}
	
	public final void teleport(Position pos) {
		remove();
		pos.room().addObject(this, pos);
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
	
	public final <T> Optional<T> findNearest(OptionalFunction<GameObject, T> mapper) {
		return getRoom().objectStream().sorted(blockingComparator().thenComparing(Room.byProximity(getCoord())))
				.flatMap(mapper.toStream()).findFirst();
	}
	
	public final void remove() {
		getRoom().removeObject(this);
		position = null;
	}
	
	private Comparator<GameObject> blockingComparator() {
		return Comparator.comparing(value -> isAccessBlocked(value.getCoord()), Boolean::compareTo);
	}
}