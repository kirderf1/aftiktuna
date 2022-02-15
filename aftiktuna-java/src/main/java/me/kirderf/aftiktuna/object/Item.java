package me.kirderf.aftiktuna.object;

import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

public final class Item extends GameObject {
	public static final OptionalFunction<GameObject, Item> CAST = OptionalFunction.cast(Item.class);
	
	private final ItemType type;
	
	public Item(ItemType type) {
		super(type, 1);
		this.type = type;
		
		if (!ObjectTypes.ITEMS.contains(type))
			throw new IllegalArgumentException("Invalid item type %s".formatted(type.name()));
	}
	
	@Override
	public ItemType getType() {
		return type;
	}
}
