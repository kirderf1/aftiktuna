package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.object.WeaponType;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Comparator;
import java.util.Optional;

public final class WieldTask extends Task {
	private final Aftik aftik;
	
	public WieldTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		Optional<WeaponType> weaponType = findWieldableInventoryItem();
		
		if (weaponType.isPresent()) {
			aftik.wieldFromInventory(weaponType.get(), out);
			return true;
		} else
			return false;
	}
	
	private Optional<WeaponType> findWieldableInventoryItem() {
		int currentWeaponValue = aftik.getWieldedItem().map(WeaponType::getDamageValue).orElse(0);
		return aftik.getInventory().stream().flatMap(OptionalFunction.cast(WeaponType.class).toStream())
				.max(Comparator.comparingInt(WeaponType::getDamageValue))
				.filter(type -> currentWeaponValue < type.getDamageValue());
	}
}