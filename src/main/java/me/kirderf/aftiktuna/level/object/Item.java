package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.util.OptionalFunction;

public class Item extends GameObject {
	public static final OptionalFunction<GameObject, Item> CAST = OptionalFunction.cast(Item.class);
	
	public Item(ObjectType type) {
		super(type, 1);
	}
}
