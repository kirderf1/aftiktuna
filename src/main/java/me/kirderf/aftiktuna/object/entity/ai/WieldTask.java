package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.WeaponType;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Comparator;
import java.util.Optional;

/**
 * A task where the character wields their best weapon in their inventory.
 */
public final class WieldTask extends Task {
	private final Aftik aftik;
	
	public WieldTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ActionPrinter out) {
		Optional<WeaponType> weaponType = findWieldableInventoryItem(aftik);
		
		if (weaponType.isPresent()) {
			aftik.wieldFromInventory(weaponType.get(), out);
			return true;
		} else
			return false;
	}
	
	public static Optional<WeaponType> findWieldableInventoryItem(Aftik aftik) {
		int currentWeaponValue = aftik.getWieldedItem().map(WeaponType::getDamageValue).orElse(0);
		
		return aftik.getInventory().stream().flatMap(OptionalFunction.cast(WeaponType.class).toStream())
				.max(Comparator.comparingInt(WeaponType::getDamageValue))
				.filter(type -> currentWeaponValue < type.getDamageValue());
	}
}