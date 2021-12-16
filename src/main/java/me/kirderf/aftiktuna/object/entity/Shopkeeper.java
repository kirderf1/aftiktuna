package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.type.ItemType;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.List;

public final class Shopkeeper extends GameObject {
	public static final OptionalFunction<GameObject, Shopkeeper> CAST = OptionalFunction.cast(Shopkeeper.class);
	
	private final List<ItemType> stock;
	
	public Shopkeeper(ItemType... items) {
		super(ObjectTypes.SHOPKEEPER, 15);
		stock = List.of(items);
	}
	
	public List<ItemType> getItemsInStock() {
		return stock;
	}
	
	public boolean buyItem(Crew crew, ItemType item) {
		return getItemsInStock().contains(item) &&
				crew.trySpendPoints(item.getPrice());
	}
}