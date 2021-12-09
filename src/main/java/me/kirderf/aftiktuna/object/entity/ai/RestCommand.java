package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Entity;
import me.kirderf.aftiktuna.print.ActionPrinter;

public final class RestCommand extends Command {
	private final Aftik aftik;
	
	public RestCommand(Aftik aftik) {
		this.aftik = aftik;
	}
	
	@Override
	public Status prepare() {
		if (aftik.getArea().objectStream().flatMap(Aftik.CAST.toStream()).allMatch(Entity::isRested)) {
			return Status.REMOVE;
		} else {
			return Status.KEEP;
		}
	}
	
	@Override
	public Status performAction(ActionPrinter out) {
		return Status.KEEP;
	}
}