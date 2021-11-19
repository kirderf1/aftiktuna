package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.print.ActionPrinter;

public abstract class Command {
	
	/**
	 * Returns REMOVE when the command is finished and should be removed.
	 */
	public abstract Status performAction(ActionPrinter out);
	
	public enum Status {
		KEEP,
		REMOVE
	}
}