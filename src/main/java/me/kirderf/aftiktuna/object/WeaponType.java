package me.kirderf.aftiktuna.object;

public final class WeaponType extends ItemType {
	private final int damageValue;
	
	public WeaponType(char symbol, String name, int damageValue, int price, String examineText) {
		super(symbol, name, price, examineText);
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