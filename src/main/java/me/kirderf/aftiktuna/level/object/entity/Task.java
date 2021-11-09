package me.kirderf.aftiktuna.level.object.entity;

import me.kirderf.aftiktuna.ContextPrinter;
import me.kirderf.aftiktuna.action.result.EnterResult;
import me.kirderf.aftiktuna.level.object.door.Door;

public abstract class Task {
	public abstract boolean performAction(ContextPrinter out);
	
	public void prepare() {}
	
	public void observeEnteredDoor(Aftik aftik, Door door, EnterResult result) {}
}