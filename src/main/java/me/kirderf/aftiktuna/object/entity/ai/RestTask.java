package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class RestTask extends Task {
	private final Aftik aftik;
	
	public RestTask(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public Status prepare() {
		if (isAllRested(aftik) || !isAreaSafe(aftik)) {
			return Status.REMOVE;
		} else {
			return Status.KEEP;
		}
	}
	
	@Override
	public Status performAction(ActionPrinter out) {
		return Status.KEEP;
	}
	
	public static boolean isAllRested(Aftik aftik) {
		return aftik.getArea().objectStream().flatMap(Aftik.CAST.toStream()).allMatch(Entity::isRested);
	}
	
	public static boolean isAreaSafe(Aftik aftik) {
		return aftik.getArea().objectStream().noneMatch(Creature.CAST.toPredicate());
	}
}