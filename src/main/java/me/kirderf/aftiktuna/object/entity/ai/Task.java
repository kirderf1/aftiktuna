package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;

public abstract class Task {
	public abstract boolean performAction(ContextPrinter out);
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {}
}