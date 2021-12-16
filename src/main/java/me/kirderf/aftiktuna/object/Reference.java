package me.kirderf.aftiktuna.object;

import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;

import java.util.Optional;

public final class Reference<T extends GameObject> {
	
	private final Identifier id;
	private final Class<T> clazz;
	
	public Reference(T object, Class<T> clazz) {
		this.id = object.getId();
		this.clazz = clazz;
	}
	
	public Optional<T> find(Area area) {
		return area.objectStream().filter(obj -> obj.getId().equals(id)).findAny().map(clazz::cast);
	}
	
	public boolean isIn(Area area) {
		return find(area).isPresent();
	}
	
	public T getOrThrow(Area area) {
		return find(area).orElseThrow();
	}
}