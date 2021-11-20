package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;

public final class Shopkeeper extends GameObject {
	public static final OptionalFunction<GameObject, Shopkeeper> CAST = OptionalFunction.cast(Shopkeeper.class);
	
	public Shopkeeper() {
		super(ObjectTypes.SHOPKEEPER, 15);
	}
	
	public Optional<ObjectType> buyItem(Crew crew) {
		if (crew.trySpendPoints(ObjectTypes.FUEL_CAN.getPrice())) {
			return Optional.of(ObjectTypes.FUEL_CAN);
		} else {
			return Optional.empty();
		}
	}
}