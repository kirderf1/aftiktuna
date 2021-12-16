package me.kirderf.aftiktuna.object;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.location.GameObject;

import java.util.Optional;
import java.util.stream.Stream;

public final class Reference<T extends GameObject> {
	
	private final Identifier id;
	private final Class<T> clazz;
	
	public Reference(T object, Class<T> clazz) {
		this.id = object.getId();
		this.clazz = clazz;
	}
	
	public Optional<T> find(Area area) {
		return find(area.objectStream());
	}
	
	public Optional<T> find(Stream<? extends GameObject> stream) {
		return stream.filter(obj -> obj.getId().equals(id)).findAny().map(clazz::cast);
	}
	
	public boolean isIn(Area area) {
		return find(area).isPresent();
	}
	
	public T getOrThrow(Area area) {
		return find(area).orElseThrow();
	}
	
	public T getOrThrow(Crew crew) {
		return find(crew.getCrewMembers().stream()).orElseThrow();
	}
	
	@Override
	public String toString() {
		return "Reference{" + id + '}';
	}
}