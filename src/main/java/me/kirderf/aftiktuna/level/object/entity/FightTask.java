package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;

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