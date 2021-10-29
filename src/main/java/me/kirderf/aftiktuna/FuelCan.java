package me.kirderf.aftiktuna;

public class FuelCan extends GameObject {
	public FuelCan() {
		super('f', "Fuel can");
	}
	
	@Override
	public boolean isFuelCan() {
		return true;
	}
}
