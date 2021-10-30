package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

import java.util.ArrayList;
import java.util.List;

public class Aftik extends GameObject {
	private final List<ObjectType> inventory = new ArrayList<>();
	
	public Aftik() {
		super(ObjectType.AFTIK, 10);
	}
	
	public void addItem(ObjectType type) {
		inventory.add(type);
	}
	
	public boolean hasItem(ObjectType type) {
		return inventory.contains(type);
	}
}