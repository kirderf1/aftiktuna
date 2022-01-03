package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.door.DoorProperty;

public final class WeaponType extends ItemType {
	private final int damageValue;
	
	public WeaponType(char symbol, String name, String pluralName, int damageValue, int price, String examineText) {
		this(symbol, name, pluralName, damageValue, price, null, examineText);
	}
	
	public WeaponType(char symbol, String name, String pluralName, int damageValue, int price, DoorProperty.Method forceMethod, String examineText) {
		super(symbol, name, pluralName, price, forceMethod, examineText);
		this.damageValue = damageValue;
	}
	
	public int getDamageValue() {
		return damageValue;
	}
	
	@Override
	public String toString() {
		return "WeaponType{" +
				"symbol=" + symbol +
				", name='" + name + '\'' +
				", damageValue=" + damageValue +
				'}';
	}
}