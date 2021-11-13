package me.kirderf.aftiktuna.object;

public final class WeaponType extends ObjectType {
	private final int damageValue;
	
	public WeaponType(char symbol, String name, int damageValue) {
		super(symbol, name);
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