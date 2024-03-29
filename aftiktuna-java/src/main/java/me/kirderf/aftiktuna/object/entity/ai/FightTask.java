package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

/**
 * A task where the character attacks a nearby creature.
 */
public final class FightTask extends StaticTask {
	private final Aftik aftik;
	
	public FightTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ActionPrinter out) {
		Optional<Creature> target = aftik.findNearestAccessible(Creature.CAST, false);
		if (target.isPresent()) {
			aftik.moveAndAttack(target.get(), out);
			return true;
		} else
			return false;
	}
}