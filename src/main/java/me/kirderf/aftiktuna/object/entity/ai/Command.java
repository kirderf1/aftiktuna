package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.print.ContextPrinter;

public abstract class Command {
	
	/**
	 * Returns true when the command is finished and should be removed.
	 */
	public abstract boolean performAction(ContextPrinter out);
}