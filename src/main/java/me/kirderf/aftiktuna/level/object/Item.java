package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;

public class Item extends GameObject {
	public Item(ObjectType type) {
		super(type, 1);
	}
	
	@Override
	public boolean isItem() {
		return true;
	}
}
