package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.print.ContextPrinter;

import java.util.Optional;

public final class FightTask extends Task {
	private final Aftik aftik;
	
	public FightTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public boolean performAction(ContextPrinter out) {
		Optional<Creature> target = aftik.findNearest(Creature.CAST);
		if (target.isPresent()) {
			aftik.moveAndAttack(target.get(), out);
			return true;
		} else
			return false;
	}
}