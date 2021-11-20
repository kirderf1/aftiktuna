package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.ItemType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.List;

public final class Shopkeeper extends GameObject {
	public static final OptionalFunction<GameObject, Shopkeeper> CAST = OptionalFunction.cast(Shopkeeper.class);
	
	public Shopkeeper() {
		super(ObjectTypes.SHOPKEEPER, 15);
	}
	
	public List<ItemType> getItemsInStock() {
		return List.of(ObjectTypes.FUEL_CAN);
	}
	
	public boolean buyItem(Crew crew, ItemType item) {
		return getItemsInStock().contains(item) &&
				crew.trySpendPoints(item.getPrice());
	}
}