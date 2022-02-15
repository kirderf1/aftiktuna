package me.kirderf.aftiktuna.object;

import java.util.Objects;

public final class Identifier<T> {
	private static int nextId;
	
	public static <T> Identifier<T> newId() {
		return new Identifier<>(nextId++);
	}
	
	private final int id;
	
	private Identifier(int id) {
		this.id = id;
	}
	
	@Override
	public String toString() {
		return "Id{" + id + '}';
	}
	
	@Override
	public boolean equals(Object o) {
		if (this == o) return true;
		if (o == null || getClass() != o.getClass()) return false;
		Identifier<?> identifier = (Identifier<?>) o;
		return id == identifier.id;
	}
	
	@Override
	public int hashCode() {
		return Objects.hash(id);
	}
}