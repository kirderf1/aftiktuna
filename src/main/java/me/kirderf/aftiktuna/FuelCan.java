package me.kirderf.aftiktuna;

import me.kirderf.aftiktuna.level.GameObject;

public class FuelCan extends GameObject {
	public FuelCan() {
		super('f', "Fuel can", 1);
	}
	
	@Override
	public boolean isFuelCan() {
		return true;
	}
}
