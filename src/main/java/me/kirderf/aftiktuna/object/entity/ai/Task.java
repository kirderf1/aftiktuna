package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ContextPrinter;

public abstract class Task {
	public abstract boolean performAction(ContextPrinter out);
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {}
}