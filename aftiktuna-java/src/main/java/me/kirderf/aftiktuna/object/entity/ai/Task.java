package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

public abstract class Task {
	
	public Status prepare(Aftik aftik) {
		return Status.KEEP;
	}
	
	/**
	 * Returns REMOVE when the task is finished and should be removed.
	 */
	public abstract Status performAction(Aftik aftik, ActionPrinter out);
	
	public enum Status {
		KEEP,
		REMOVE
	}
}