package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.print.ActionPrinter;

public abstract class Task {
	
	public Status prepare() {
		return Status.KEEP;
	}
	
	/**
	 * Returns REMOVE when the command is finished and should be removed.
	 */
	public abstract Status performAction(ActionPrinter out);
	
	public enum Status {
		KEEP,
		REMOVE
	}
}