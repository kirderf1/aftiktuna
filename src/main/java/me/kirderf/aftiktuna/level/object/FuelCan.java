package me.kirderf.aftiktuna.level.object;

import me.kirderf.aftiktuna.level.GameObject;
import me.kirderf.aftiktuna.level.ObjectType;

public class FuelCan extends GameObject {
	public FuelCan() {
		super(ObjectType.FUEL_CAN, 1);
	}
	
	@Override
	public boolean isFuelCan() {
		return true;
	}
}
